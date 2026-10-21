# Offline tests

`core_test.c` executes only the new protocol core, linked with local fake HAL
and callback functions. It never links a real adapter or executes the RAM image.
`golden_vectors.h` holds literal independent CBW/reply examples.

From the repository root, using the installed host C compiler:

```sh
rtk proxy mkdir -p target/fmr-tests
rtk proxy clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -DFMR_HOST_TEST -DFMR_BUILD_ID=0x12345678 -Iram-reader/include \
  ram-reader/src/reader.c ram-reader/tests/core_test.c \
  -o target/fmr-tests/core-test
rtk proxy target/fmr-tests/core-test
rtk proxy clang -std=c11 -Wall -Wextra -Werror -fsanitize=address,undefined \
  -DFMR_HOST_TEST -DFMR_HOST_MMIO_TEST -Iram-reader/include \
  ram-reader/src/wl82.c ram-reader/tests/mmio_test.c \
  -o target/fmr-tests/mmio-test
rtk proxy target/fmr-tests/mmio-test
```

The test build ID is deliberately recognizable; it does not designate the
target artifact. The harness checks identity/JEDEC/read vectors, exact direction
and size, canonical reserved bytes, integer bounds/alignment, every forbidden
16-bit command, permanent failure, dirty initial state, pointer ownership,
reentrant callbacks, fresh content, DMA counts and guard canaries. A fake sender
return value never means successful USB delivery. Host sanitizers cannot prove
ROM callback consumption, live RAM ownership, MMIO timing or physical SPI/NOR
behavior. No device access, firmware scripts or target execution is required.

`mmio_test.c` executes the same HAL source used in the target with every register
read/write redirected to local storage. It checks exact JEDEC9F and raw03
transactions, failure at each of four JEDEC and five read completion phases,
the shared finite polling budget, DMA address/count/raw-mode configuration,
defensive bounds and quiescence. The fake assumes pending-bit/W1C semantics;
it never dereferences the truncated target DMA address or simulates NOR content.
Payload freshness/content belongs to the separate core fake test. These tests
establish code behavior under injected assumptions, not physical qualification.
