#!/bin/bash

# MY_SECRET_PASS should be set as env var in CI
echo ">>> values dev <<<"
echo ""
complate -e render -t chart -v "version=0.1.0"
echo ">>> values dev <<<"
echo ""
MY_SECRET_PASS="pass:superpass" complate -e render -t dev --trust
echo ""
echo ">>> values prod <<<"
echo ""
MY_SECRET_PASS="pass:superpass" complate -e render -t prod --trust
