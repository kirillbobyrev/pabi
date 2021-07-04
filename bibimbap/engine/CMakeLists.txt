add_executable(bibimbap driver.cc)

target_link_libraries(bibimbap Chess absl::flags absl::flags_parse)
