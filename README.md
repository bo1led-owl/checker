# Checker
A little checker for competitive programming solutions

## Usage
    checker [OPTIONS] <TESTS> <SOLUTION>

To see detailed options use:

    checker --help

## Test suite description syntax
A test is described after `[test]` header, its input - after `[input]` header and the answer - after `[answer]` header. Both input and answer can be multiline.

For example, if the solution takes two integers and prints their sum, the test suite can be described as such:

    [test]
    [input]
    1 2
    [answer]
    3
    
    [test]
    [input]
    7 2
    [answer]
    9

    [test]
    [input]
    16 16
    [answer]
    32
