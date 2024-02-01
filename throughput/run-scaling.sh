cargo build #--release
n=0
while [ "$n" -lt 50 ]; do
    n=$(( n + 1 ))
    ./target/release/star-vs-chain-execution \
	--depth 1000 \
    --iterations 50 \
    --remote 10.0.1.2:1234 \
    --scaling \
	client \
	--interface veth3 \
	--switch-addr 10.0.9.2:1234 \
	--address 10.0.3.2:1234
    sleep 3
done
