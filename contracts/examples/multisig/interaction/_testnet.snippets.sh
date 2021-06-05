JSON="/home/dragos/erd1zxg76acu90nr4qqtkf5eahmqpy82dptps2lnzrjfp73scpr0306qqtnfxv.json"
PASS="/home/dragos/erd1zxg76acu90nr4qqtkf5eahmqpy82dptps2lnzrjfp73scpr0306qqtnfxv.pass"
ADDRESS=$(erdpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)
#PROXY=https://testnet-api.elrond.com
ALICE="${USERS}/alice.pem"

# 
deploy() {
    erdpy --verbose contract deploy --project=${PROJECT} \
          --recall-nonce --keyfile=${JSON} --passfile=${PASS} \
          --gas-limit=500000000 \
          --arguments 1 0x1191ed771c2be63a800bb2699edf60090ea6856182bf310e490fa30c046f8bf4 \
          --send --outfile="deploy-testnet.interaction.json" \

    TRANSACTION=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

getNumBoardMembers() {
    erdpy --verbose contract query ${ADDRESS} --function="getNumBoardMembers" 
}
getNumProposers() {
    erdpy --verbose contract query ${ADDRESS} --function="getNumProposers" 
}
getPendingActionCount() {
    erdpy --verbose contract query ${ADDRESS} --function="getPendingActionCount" 
}
getActionLastIndex() {
    erdpy --verbose contract query ${ADDRESS} --function="getActionLastIndex" 
}
getActionData() {
    read -p 'Action:' ACTION
    erdpy --verbose contract query ${ADDRESS} --function="getActionData"  --arguments ${ACTION}
}

getQuorum() {
    erdpy --verbose contract query ${ADDRESS} --function="getQuorum" 
}

getUserRole() {
    erdpy --verbose contract query ${ADDRESS} --function="getUserRole" 
}

getTokenIdentifier(){
    erdpy --verbose contract query ${ADDRESS} --function="getTokenIdentifier"
}

proposeAddBoardMember() {
    read -p 'Address:' MEMBER
    erdpy --verbose contract query ${ADDRESS} --function="proposeAddBoardMember" --arguments ${MEMBER} 
}
userRole() {
    read -p 'Address:' MEMBER
    erdpy --verbose contract query ${ADDRESS} --function="userRole" --arguments ${MEMBER} 
}

callback_data_at_index() {
    read -p 'Index:' INDEX
    erdpy --verbose contract query ${ADDRESS} --function="callback_data_at_index" --arguments ${INDEX} 
}
#0x000000000000000000010000000000000000000000000000000000000002ffff erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u
#5000696200000000000
#0x697373756540353437323735373337343533373436313662363936653637353234664035343533353234664032373130403030403633363136653535373036373732363136343635403734373237353635 issue@54727573745374616b696e67524f@5453524f@01@00@63616e55706772616465@74727565
proposeSendEgld() {
    read -p 'Address:' SC 
    read -p 'Data:' Data
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --keyfile=${JSON} --passfile=${PASS} --gas-limit=50000000 --function="proposeSendEgld" --arguments ${SC} 5000000000000000000 ${Data} --send  --chain=T
}

proposeSendEgld() {
    read -p 'Address:' SC 
    read -p 'Data:' Data
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --keyfile=${JSON} --passfile=${PASS} --gas-limit=50000000 --function="proposeSendEgld" --arguments ${SC} 5000000000000000000 ${Data} --send  --chain=T
}

deposit() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --keyfile=${JSON} --passfile=${PASS} --gas-limit=50000000 --function="deposit" --value 6000000000000000000 --send  --chain=T
}

proposeSCCall() {
    # read -p 'Address:' SC 
    # read -p 'Fee:' FEE
    # read -p 'Args:' ARGS 
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --keyfile=${JSON} --passfile=${PASS} --gas-price=1000000000 --gas-limit=50000000 --function=proposeSCCall --send --arguments 0x000000000000000000010000000000000000000000000000000000000002ffff 5000000000000000000 0x697373756540353437323735373337343533373436313662363936653637353234664035343533353234664032373130403030403633363136653535373036373732363136343635403734373237353635
}
issueScToken() {
    # read -p 'Address:' SC 
    # read -p 'Fee:' FEE
    # read -p 'Args:' ARGS 
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --keyfile=${JSON} --passfile=${PASS} \
        --gas-price=1000000000 --gas-limit=1000000000 --function=issueScToken --send \
        --arguments 0x54727573745374616b696e67524f 0x5453524f 10000 \
        --value 6000000000000000000
}

performAction() {
    read -p 'action:' SC 
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --keyfile=${JSON} --passfile=${PASS} --gas-limit=1000000000 --function="performAction" --arguments ${SC} --send  --chain=T
}

getSum() {
    erdpy --verbose contract query ${ADDRESS} --function="getSum" 
}
