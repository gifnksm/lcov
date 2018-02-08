#include <stdio.h>

#include "main.h"

int add(int x, int y)
{
  return x + y;
}

int sub(int x, int y)
{
  return x - y;
}

int main(void)
{
  int sum = 0;
  int n = 10;

  for (int i = 0; i < n; i++) {
    sum = add(sum, i);
  }

  printf("sum=%d\n", sum);

  if ((sum % 10) == 0) {
    printf("sum is multiple of 10\n");
  }
  if (sum == mul(5, 11)) {
    printf("sum is 5 * 11\n");
  }
}
