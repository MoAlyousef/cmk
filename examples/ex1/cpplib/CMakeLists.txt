cmake_minimum_required(VERSION 3.14)
project(cpplib)

add_library(cpplib main.cpp)

install(TARGETS cpplib ARCHIVE DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)