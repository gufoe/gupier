set -e
HOST=$1
#rsync --progress target/x86_64-unknown-linux-musl/release/gupier $HOST:~/
ssh -t $HOST ~/gupier -s 20016
