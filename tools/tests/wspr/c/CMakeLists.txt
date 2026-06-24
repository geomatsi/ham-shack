cmake_minimum_required(VERSION 3.14)
project(wspr C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_C_EXTENSIONS OFF)

add_compile_options(-Wall -Wextra -Wpedantic)

add_library(wspr STATIC wspr.c)
target_include_directories(wspr PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})

add_executable(wspr_app main.c)
target_link_libraries(wspr_app PRIVATE wspr)

enable_testing()

include(FetchContent)
FetchContent_Declare(
	unity
	GIT_REPOSITORY https://github.com/ThrowTheSwitch/Unity.git
	GIT_TAG        v2.6.0
)
FetchContent_MakeAvailable(unity)

add_executable(test_f1 tests/test_f1.c)
target_link_libraries(test_f1 PRIVATE wspr unity)
add_test(NAME f1 COMMAND test_f1)

add_executable(test_f2 tests/test_f2.c)
target_link_libraries(test_f2 PRIVATE wspr unity)
add_test(NAME f2 COMMAND test_f2)
