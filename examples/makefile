
TARGETS := fib timer_ticks echo_spin echo_interrupt threads

TARGETS := $(addprefix build/,$(TARGETS))
AS := ../target/debug/as_cli
BUILD_DIR = build

.PHONY: all clean
all: $(TARGETS)

$(BUILD_DIR):
	mkdir $(BUILD_DIR)

$(AS):
	cargo +nightly -C .. build --bin as_cli

$(BUILD_DIR)/fib: fib.s teletype_spin.s $(AS) | $(BUILD_DIR)
$(BUILD_DIR)/timer_ticks: timer_ticks.s teletype_spin.s $(AS) | $(BUILD_DIR)
$(BUILD_DIR)/echo_spin: echo_spin.s teletype_spin.s $(AS) | $(BUILD_DIR)
$(BUILD_DIR)/echo_interrupt: echo_interrupt.s byte_queue.s $(AS) | $(BUILD_DIR)
$(BUILD_DIR)/threads: threads.s $(AS) | $(BUILD_DIR)

$(TARGETS):
	cat $(filter %.s,$^) | $(AS) - -o $@

clean:
	@rm -rf build/ || true

