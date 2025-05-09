# Checker
A little checker for competitive programming solutions

## Usage
To run tests use `run` command:

    checker run ./solution # for executable without arguments
    checker run "python solution.py" # for more complex commands

To add a test use `add-test` (the first argument is the input, the second one is the answer):

    checker add-test "1 2" "3"

To clear the test suite use `clear-tests`:

    checker clear-tests

By default, test suite is stored in `./tests`, but this behaivour can be overridden by specifying the path after `--test-suite`

For more details use `checker help` or `checker --help`

## Test suite description syntax
A test is described after `[test]` header, its input - after `[in]` header and the answer - after `[out]` header. Both input and answer can be multiline.

For example, if the solution takes two integers and prints their sum, the test suite can be described like this:

    [test]
    [in]
    1 2
    [out]
    3
    
    [test]
    [in]
    7 2
    [out]
    9

    [test]
    [in]
    16 16
    [out]
    32
