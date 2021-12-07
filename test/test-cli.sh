#!/bin/bash

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
echo "success"

echo "Testing with ultimate shell-trust"
cmd="../target/debug/complate -e render -c=./config.yml -t=test --shell-trust=ultimate"
$cmd
ev 0 $?
echo
echo "success"

echo "Testing value overrides"
cmd="../target/debug/complate -e render -c=./config.yml -t=test -v=alpha=bananarama"
$cmd
ev 0 $?
echo
echo "success"

echo ""
echo "All tests succeeded."
echo ""
exit 0
