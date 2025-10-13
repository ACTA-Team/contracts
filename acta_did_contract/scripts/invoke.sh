account_1="GDIY6AQQ75WMD4W46EYB7O6UYMHOCGQHLAQGQTKHDX4J2DYQCHVCR4W4"
account_2="GAICHJM4OUNAVKALCO2ANSXVSOD7Z2UTXE55R5RY3RX352LSJC6SYZXV" # test
account_3="GCITMPKRIPOP4VV5IP5XGYKRNKYOVHCJMPNTFGQNAEJ5WW2PVCDWGR6B" # second_account

# contract_id=CANZK4ELDKWKHW2NDQFCZ7LE5G7W23RDNIFEOIQDZ45ACMW7F576BGFB # Standalone
contract_id=CADVHUKFQ5HF5SANSUARZO5QBGZSCYFFLICDCY5T2PHYLFQA32SA5QNG # Futurenet
# contract_id=CBSQC365UCBQZJNS75NF6VIIW3PJUQ3DYPLWODVC2OGVPMJAIRJTDOS4 # Testnet

NETWORK=testnet
# soroban config network add --global testnet \
#     --rpc-url https://soroban-testnet.stellar.org:443 \
#     --network-passphrase "Test SDF Network ; September 2015"
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

# soroban contract invoke --id $contract_id --source test --network $NETWORK -- identity --user $account_1
soroban contract invoke --id $(cat .soroban/$NETWORK) --source test --network $NETWORK -- identity --user $account_2
# soroban contract invoke --id $(cat .soroban/$NETWORK) --source test --network $NETWORK -- hello --user $account_2

#  if you run into Missing signing key for account then you are using wrong source
# soroban contract invoke --id $(cat .soroban/$NETWORK) --network $NETWORK -- transfer --id $account_2 --from $account_1 --to $account_2
# soroban contract invoke --id $(cat .soroban/$NETWORK) --network $NETWORK --source test -- transfer --id $account_2 --from $account_2 --to $account_1
# soroban contract invoke --id $(cat .soroban/$NETWORK) --network $NETWORK --source second_account -- transfer --id $account_2 --from $account_3 --to $account_2
