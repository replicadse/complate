#!/bin/sh

ev() {
    if [ $2 -eq $1 ]
    then
        printf ""
    else
        printf "command failed\n" && exit 1
    fi
}

echo "Testing with no shell trust"
cmd="../target/debug/complate -e render -c=./config.yml -t=test"
$cmd
ev 1 $?

echo "Testing with ultimate shell-trust"
cmd="../target/debug/complate -e render -c=./config.yml -t=test --shell-trust=ultimate"
$cmd
ev 0 $?
echo ""

echo ""
echo "All tests succeeded."
echo ""
exit 0
