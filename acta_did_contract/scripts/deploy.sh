account_1="GDIY6AQQ75WMD4W46EYB7O6UYMHOCGQHLAQGQTKHDX4J2DYQCHVCR4W4"
account_2="GAICHJM4OUNAVKALCO2ANSXVSOD7Z2UTXE55R5RY3RX352LSJC6SYZXV" # test
account_3="GCITMPKRIPOP4VV5IP5XGYKRNKYOVHCJMPNTFGQNAEJ5WW2PVCDWGR6B" # second_account

NETWORK=testnet
soroban config network add --global testnet \
    --rpc-url https://soroban-testnet.stellar.org:443 \
    --network-passphrase "Test SDF Network ; September 2015"
# curl "https://friendbot.stellar.org/?addr=$account_1"
# curl "https://friendbot.stellar.org/?addr=$account_2"
# curl "https://friendbot.stellar.org/?addr=$account_3"

# NETWORK=futurenet
# soroban config network add --global futurenet \                                                                                                                                                                                      6s Py base 11:37:37 AM
#   --rpc-url https://rpc-futurenet.stellar.org:443 \
#   --network-passphrase "Test SDF Future Network ; October 2022"

# curl "https://friendbot-futurenet.stellar.org/?addr=$account_1"
# curl "https://friendbot-futurenet.stellar.org/?addr=$account_2"
# curl "https://friendbot-futurenet.stellar.org/?addr=$account_3"

# NETWORK=localhost
#  soroban config network add --global localhost \                                                                                                                                                                                         Py base 11:39:50 AM
#   --rpc-url http://localhost:8000/soroban/rpc \
#   --network-passphrase "Standalone Network ; February 2017"
# curl "http://localhost:8000/friendbot?addr=$account_1"
# curl "http://localhost:8000/friendbot?addr=$account_2"
# curl "http://localhost:8000/friendbot?addr=$account_3"

cargo build --target wasm32-unknown-unknown --release
soroban contract optimize \
    --wasm target/wasm32-unknown-unknown/release/stellar_did_registry.wasm
HASH=$(soroban contract install --wasm target/wasm32-unknown-unknown/release/stellar_did_registry.optimized.wasm --network $NETWORK)
echo $HASH
soroban contract deploy --wasm-hash $HASH --network $NETWORK --source test >.soroban/$NETWORK
echo $(cat .soroban/$NETWORK)
