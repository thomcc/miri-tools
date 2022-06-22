exec 2>&1

export TERM=xterm-256color

while read crate;
do
    cd /root/build
    find /root/build -mindepth 1 -delete # clean out anything from an old build (probably)
    if cargo download $crate /root/build
    then
        ARGS=$(python3 /root/get-args.py $crate)
        cargo +nightly update --color=always
        cargo +nightly test -Zbuild-std --target=x86_64-unknown-linux-gnu --no-run --color=always --jobs=1 $ARGS
        unbuffer -p /usr/bin/time -v cargo +nightly nextest run -Zbuild-std --target=x86_64-unknown-linux-gnu --no-fail-fast --config-file=/root/.cargo/nextest.toml --jobs=1 $ARGS
        unbuffer -p /usr/bin/time -v timeout $TEST_TIMEOUT cargo +nightly test -Zbuild-std --target=x86_64-unknown-linux-gnu --doc --no-fail-fast --jobs=1 $ARGS
        cat Cargo.lock
    fi
    echo "-${TEST_END_DELIMITER}-"
done < /dev/stdin
