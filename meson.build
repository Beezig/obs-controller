project('obs-controller', 'cpp', license: 'GPLv3+', version: '0.1.0')

# Dependencies
libobs = dependency('libobs', version: '>=25.0.0')

# Include paths
inc = include_directories(['/usr/include/obs'])

library('obs-controller', ['src/main.cpp', 'src/api.cpp'], dependencies: [libobs], include_directories: inc)