#!/bin/bash

if [ "$1" = "" ]
then
  echo "Usage: $0 1 arg required - juno address"
  exit
fi

# pinched and adapted from DA0DA0 + whoami
IMAGE_TAG=${2:-"v2.1.0"}
CONTAINER_NAME="juno_posts"
BINARY="docker exec -i $CONTAINER_NAME junod"
DENOM='ujunox'
CHAIN_ID='testing'
RPC='http://localhost:26657/'
TXFLAG="--gas-prices 0.1$DENOM --gas auto --gas-adjustment 1.3 -y -b block --chain-id $CHAIN_ID --node $RPC"
BLOCK_GAS_LIMIT=${GAS_LIMIT:-100000000} # should mirror mainnet

echo "Building $IMAGE_TAG"
echo "Configured Block Gas Limit: $BLOCK_GAS_LIMIT"

# kill any orphans
docker kill $CONTAINER_NAME
docker volume rm -f junod_data

# run junod setup script
docker run --rm -it \
    -e PASSWORD=xxxxxxxxx \
    -e STAKE_TOKEN=$DENOM \
    -e GAS_LIMIT="$GAS_LIMIT" \
    --mount type=volume,source=junod_data,target=/root \
    ghcr.io/cosmoscontracts/juno:$IMAGE_TAG /opt/setup_junod.sh $1

# we need app.toml and config.toml to enable CORS
# this means config wrangling required
docker run -v junod_data:/root --name helper busybox true
docker cp docker/app.toml helper:/root/.juno/config/app.toml
docker cp docker/config.toml helper:/root/.juno/config/config.toml
docker rm helper

docker run --rm -d --name $CONTAINER_NAME \
    -p 1317:1317 -p 26656:26656 -p 26657:26657 \
    --mount type=volume,source=junod_data,target=/root \
    ghcr.io/cosmoscontracts/juno:$IMAGE_TAG ./run_junod.sh

# compile
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.4

# copy wasm to docker container
docker cp artifacts/cw_posts.wasm $CONTAINER_NAME:/cw_posts.wasm

# validator addr
VALIDATOR_ADDR=$($BINARY keys show validator --address)
echo "Validator address:"
echo $VALIDATOR_ADDR

BALANCE_1=$($BINARY q bank balances $VALIDATOR_ADDR)
echo "Pre-store balance:"
echo $BALANCE_1

# you ideally want to run locally, get a user and then
# pass that addr in here
echo "Address to deploy contracts: $1"
echo "TX Flags: $TXFLAG"

# upload posts wasm
# CONTRACT_RES=$($BINARY tx wasm store "/cw_posts.wasm" --from validator $TXFLAG --output json)
# echo $CONTRACT_RES
CONTRACT_CODE=$($BINARY tx wasm store "/cw_posts.wasm" --from validator $TXFLAG --output json | jq -r '.logs[0].events[-1].attributes[0].value')
echo "Stored: $CONTRACT_CODE"

BALANCE_2=$($BINARY q bank balances $VALIDATOR_ADDR)
echo "Post-store balance:"
echo $BALANCE_2

# instantiate the CW721
POSTS_INIT='{
  "owner": "'"$1"'",
  "char_limit": 140,
  "post_fee": "10000"
}'
echo "$POSTS_INIT" | jq .
$BINARY tx wasm instantiate $CONTRACT_CODE "$POSTS_INIT" --from "validator" --label "posts" $TXFLAG
RES=$?

# get contract addr
CONTRACT_ADDRESS=$($BINARY q wasm list-contract-by-code $CONTRACT_CODE --output json | jq -r '.contracts[-1]')

# Print out config variables
printf "\n ------------------------ \n"
printf "Config Variables \n\n"

echo "NEXT_PUBLIC_CONTRACT_CODE=$CONTRACT_CODE"
echo "NEXT_PUBLIC_CONTRACT_ADDRESS=$CONTRACT_ADDRESS"

echo $RES
exit $RES
