#!/bin/bash

if [ -z "$AWS_ACCESS_KEY_ID" ]; then
  echo "AWS_ACCESS_KEY_ID environment variable needs to be defined"
  exit 1
fi
if [ -z "$AWS_SECRET_ACCESS_KEY" ]; then
  echo "AWS_SECRET_ACCESS_KEY environment variable needs to be defined"
  exit 1
fi

mkdir ~/.aws > /dev/null 2>&1 || true
rm ~/.aws/credentials > /dev/null 2>&1 || true
touch ~/.aws/credentials
echo "[default]" >> ~/.aws/credentials
echo "aws_access_key_id = "'$(AWS_ACCESS_KEY_ID)' >> ~/.aws/credentials
echo "aws_secret_access_key = "'$(AWS_SECRET_ACCESS_KEY)' >> ~/.aws/credentials

./near-ql
