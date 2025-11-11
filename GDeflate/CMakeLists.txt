# Copyright (c) Microsoft Corporation. All rights reserved.

cmake_minimum_required (VERSION 3.19)

project("GDeflate" VERSION 1.0.0)

add_subdirectory("GDeflate")
add_subdirectory("GDeflateDemo")

if (WIN32)
add_subdirectory("GDeflateTest")
endif (WIN32)
