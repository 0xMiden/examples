# Miden Assembly Playground

Playground for example programs for the [Miden VM](https://github.com/0xMiden/miden-vm) in Miden assembly.

[Use the playground!](https://0xmiden.github.io/examples/)

The goal of this playground is for developers to see how easy it is to write and execute code in Miden assembly. The examples come from the community and the team. If you come up with an example of your own, we are happy to include it in the list below. You can simply open a PR with your example adding the `.masm` and `.inputs` files.

Examples are written in Miden assembly, see [Miden assembly documentation](https://0xmiden.github.io/miden-docs/imported/miden-vm/src/user_docs/assembly/main.html). External inputs can be provided to the examples and the Miden VM in two ways as public inputs and via the advice provider, see [here](https://0xmiden.github.io/miden-docs/imported/miden-vm/src/intro/overview.html#inputs-and-outputs). Currently, in the playground you can use `operand_stack` as public input and the `advice_stack` as secret input.

In addition to running existing examples you can also write your own program and execute it. The examples can then serve as inspiration.

---

## Available examples

There are several examples in our repo and we hope we get more in the future. The examples range from simple to complex and aim to show how Miden assembly can be used.

### Simple examples - to see how the code works

- **Comparison**: A program which checks if the value provided as secret input via the advice tape is less than 9; if it is, the value is multiplied by 9, otherwise, 9 is added to the value; then we check if the value is odd.

- **Conditional**: A program which either adds or multiplies two numbers - 3 and 5 - based on the value provided via the advice tape as secret input.

### Math examples - see more complex numerical calculations

- **Collatz**: A program which executes an unbounded loop to compute a Collatz sequence which starts with the provided value; the output of the program is the number of steps needed to reach 1 - the end of the sequence.

- **Fibonacci**: Elegant way to calculate the 1001st fibonacci number.

- **nPrime**: Program to calculate the n-th prime number. It will return all prime numbers up to the n-th prime number which is on top of the stack.

### Complex examples - larger complex nested operations

- **Game-of-Life**: Implementation of Conway's Game of Life. But we can prove it. The static example runs on a 4x4 cell universe and 1000 generations.

---

## Testing Examples

### Quick Setup
```bash
./setup-testing.sh
```

### Test Commands
```bash
# Test all examples
cd benchmarking-cli && cargo run -- test-all

# Test specific example
cd benchmarking-cli && cargo run -- test --example fibonacci

# Validate directory structure
cd benchmarking-cli && cargo run -- validate

# Benchmark example
cd benchmarking-cli && cargo run -- benchmark --example fibonacci
```

### Running the playground locally

If you want to run it locally, then make sure you have the [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) installed.

To run the playground locally in development mode, navigate to the project directory `playground` and run:

```
npm run build:miden && npm install && npm run start
```

Open [http://localhost:3000](http://localhost:3000) to view it in the browser.

Test rust code:

```
cd miden-wasm && wasm-pack test --node
```

### Acknowledgement

We use a fork of https://github.com/timgestson/miden-assembly-playground
