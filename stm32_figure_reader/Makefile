all: node.bin

node.bin: node
	arm-none-eabi-objcopy -O binary ../target/thumbv7em-none-eabihf/release/stm32_figure_reader node.bin

node: src/*.rs
	cargo build --release
