Fibonacci in Miden Assembly: A Complete Guide
Overview
The Fibonacci example demonstrates how to compute large Fibonacci numbers efficiently in Miden Assembly. Unlike traditional iterative or recursive approaches, Miden's implementation leverages stack-based operations and loop unrolling to calculate very large sequence values (like the 1001st Fibonacci number) in a way that's both elegant and zero-knowledge friendly.
What is the Fibonacci Sequence?
The Fibonacci sequence is a series of numbers where each term is the sum of the two preceding terms:
F(0) = 0
F(1) = 1
F(n) = F(n-1) + F(n-2)  for n ≥ 2
Sequence: 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, ...
How Miden Computes Fibonacci
The Miden implementation uses an iterative doubling algorithm (also called matrix exponentiation or fast Fibonacci), which can compute the nth Fibonacci number in O(log n) time instead of O(n). This is much more efficient for large values.
The Basic Concept
Instead of computing F(1), F(2), F(3)... all the way to F(n), the algorithm uses the mathematical property that Fibonacci numbers can be computed from pairs: (F(n), F(n+1)).
Key insight: To compute F(n), the algorithm:
Represents the position n in binary
Uses doubling and addition formulas to skip ahead efficiently
Builds up the result using only the bits set in the binary representation
Stack-Based Operation
Miden Assembly works with a stack, which is a last-in-first-out (LIFO) data structure. When you push values, they sit on top; when you pop, you remove from the top.
Example:
Initial stack: []
push 1    → [1]
push 2    → [1, 2]
add       → [3]        (pops 2 and 1, adds them, pushes result)
Reading the Fibonacci Code
The Fibonacci example in Miden Assembly follows this pattern:
Input: A number n (the position in the Fibonacci sequence)
Output: Two values on the stack: F(n) and F(n+1)
Algorithm: Fast doubling using binary decomposition
Simplified pseudocode:
function fibonacci(n):
  if n == 0:
    return (0, 1)  # F(0)=0, F(1)=1
  
  # Extract the highest bit of n
  # Recursively compute F(n/2) and F((n+1)/2)
  # Double and adjust based on whether the next bit is 0 or 1
  
  return (F(n), F(n+1))
How to Use the Example
Input Format (.inputs file)
The Fibonacci example accepts input via the operand stack. The .inputs file contains a single number: the position n in the Fibonacci sequence you want to compute.
Example fibonacci.inputs:
1001
This tells the program to compute F(1001) and F(1002).
Expected Output
After execution, the Miden VM will output two values (the top two stack values):
Top of stack: F(n+1)
Second on stack: F(n)
For input 1001:
Output: F(1001) and F(1002)
F(1001) ≈ 4.3 × 10^208 (a 209-digit number!)
F(1002) ≈ 7.0 × 10^208
The Miden VM can compute this in milliseconds thanks to the efficient algorithm.
Learning the Miden Assembly Instructions
Essential Stack Instructions
Instruction
Effect
Example
push n
Push value n onto stack
push 5 → Stack: [5]
drop
Remove top value
Stack: [5, 3] → [5]
dup
Duplicate top value
Stack: [5] → [5, 5]
swap
Swap top two values
Stack: [5, 3] → [3, 5]
add
Pop two, push sum
Stack: [5, 3] → [8]
sub
Pop two, push difference
Stack: [5, 3] → [2]
mul
Pop two, push product
Stack: [5, 3] → [15]
Control Flow Instructions
Instruction
Purpose
if ... else ... end
Conditional branching
repeat ... end
Loop a fixed number of times
while ... end
Loop while condition is true
Comparison Instructions
Instruction
Effect
eq
Check equality; push 1 if equal, 0 otherwise
lt
Check less-than
gt
Check greater-than
Common Patterns in Fibonacci Code
Pattern 1: Extracting Bits
To access each bit of a number (for binary decomposition):
# Divide by 2 (right shift): use `u32div` or `div`
# Check if odd: use `u32mod 2` or a bit check
Pattern 2: Doubling Formulas
The fast Fibonacci algorithm uses these formulas to skip ahead:
F(2n) = F(n) × (2×F(n+1) - F(n))
F(2n+1) = F(n)² + F(n+1)²
These are implemented using stack operations and arithmetic in Miden Assembly.
Pattern 3: Building Results Recursively
Since you process bits from high to low, you:
Build results for the upper bits
Add contributions from lower bits as you process them
Return the final (F(n), F(n+1)) pair
Testing and Validation
Run the Example Locally
# Test the Fibonacci example
cd benchmarking-cli
cargo run -- test --example fibonacci

# Benchmark it
cargo run -- benchmark --example fibonacci
Verify Results
You can cross-check results with external Fibonacci calculators:
Online tools: Wolfram Alpha, Python's math.fib()
Reference: Miden documentation examples
Common Issues
Issue
Cause
Solution
Stack underflow
Not enough values on stack
Check that all inputs are provided
Overflow
Number too large
Fibonacci values grow exponentially; limit to F(300) for safe computation
Logic error
Incorrect doubling formulas
Review the fast doubling algorithm implementation
Why This Matters in Zero-Knowledge
The efficient Fibonacci algorithm is significant for zero-knowledge proofs because:
Compact proofs: O(log n) operations → smaller proof size
Lower computation: Fewer stack operations → less work for the prover
Scalability: Can prove knowledge of very large Fibonacci numbers without excessive overhead
Educational: Demonstrates how number theory optimizations translate to ZK efficiency
Further Exploration
Next Steps
Modify the input: Try computing F(100), F(500), F(1000)
Understand doubling: Study the binary decomposition step by step
Add checks: Extend the code to validate the relationship F(n-1) + F(n) = F(n+1)
Compare with other examples: See how Collatz or Prime examples use different patterns
Related Examples in the Repository
Fibonacci: Fast doubling algorithm (this example)
Collatz: Loop-based computation with conditional branches
nPrime: Iterative search with counter management
Game-of-Life: Complex state management and cell updates
References
Miden Assembly Documentation
Fast Fibonacci Algorithm - Detailed mathematical explanation
Zero-Knowledge Proofs & Efficiency
Quick Cheat Sheet
Input: Position n in Fibonacci sequence
Output: Stack contains [F(n), F(n+1)]
Algorithm: Binary decomposition with fast doubling
Time Complexity: O(log n) multiplications + additions
Space Complexity: O(log n) stack depth
Max Input: Practical limit ~1001 (larger values may overflow)
Did this help? If you have questions about the Fibonacci algorithm or Miden Assembly, open an issue or submit a PR with clarifications!
