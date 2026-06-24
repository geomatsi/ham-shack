#include <stddef.h>

#include "wspr.h"

uint64_t f1(void)
{
	return 42;
}

int f2(uint64_t a[], int n)
{
	if (a == NULL || n < 0)
		return -1;

	for (int i = 0; i < n; i++)
		a[i] = (uint64_t)i;

	return 0;
}
