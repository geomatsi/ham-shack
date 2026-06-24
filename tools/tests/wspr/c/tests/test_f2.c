#include "unity.h"
#include "wspr.h"

#define N 8

static uint64_t buf[N];

void setUp(void)
{
	for (int i = 0; i < N; i++)
		buf[i] = 0xDEADBEEF;
}

void tearDown(void) {}

static void test_f2_fills_sequence(void)
{
	TEST_ASSERT_EQUAL_INT(0, f2(buf, N));
	for (int i = 0; i < N; i++)
		TEST_ASSERT_EQUAL_UINT64((uint64_t)i, buf[i]);
}

static void test_f2_zero_length(void)
{
	TEST_ASSERT_EQUAL_INT(0, f2(buf, 0));
	for (int i = 0; i < N; i++)
		TEST_ASSERT_EQUAL_UINT64(0xDEADBEEF, buf[i]);
}

static void test_f2_null_array(void)
{
	TEST_ASSERT_EQUAL_INT(-1, f2(NULL, N));
}

int main(void)
{
	UNITY_BEGIN();
	RUN_TEST(test_f2_fills_sequence);
	RUN_TEST(test_f2_zero_length);
	RUN_TEST(test_f2_null_array);
	return UNITY_END();
}
