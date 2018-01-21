#include <stdio.h>

void fizzbuzz(int n) {
  if (n % 3 == 0 && n % 5 == 0) {
    printf("FizzBuzz\n");
    return;
  }
  if (n % 3 == 0) {
    printf("Fizz\n");
    return;
  }
  if (n % 5 == 0) {
    printf("Buzz\n");
    return;
  }
  printf("%d\n", n);
}
