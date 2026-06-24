#include "unity.h"
#include "wspr.h"

void setUp(void) {}
void tearDown(void) {}

static void test_f1_returns_42(void)
{
	TEST_ASSERT_EQUAL_UINT64(42, f1());
}

int main(void)
{
	UNITY_BEGIN();
	RUN_TEST(test_f1_returns_42);
	return UNITY_END();
}
