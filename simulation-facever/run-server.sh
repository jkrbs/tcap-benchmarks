cargo build # --release
# n=0
# while [ "$n" -lt 50 ]; do
#     n=$(( n + 1 ))
    cargo run  -- --debug \
	--iterations 1 \
	--transfer-size 1 \
	frontend \
	--interface veth1 \
	--switch-addr 10.0.9.2:1234 \
	--address 10.0.1.2:1234 \
	--remote 10.0.3.2:1234
# done
