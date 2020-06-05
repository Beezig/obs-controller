project('obs-controller', 'cpp', license: 'GPLv3+', version: '0.1.0')

cc = meson.get_compiler('cpp')

# Dependencies
libobs = dependency('libobs', version: '>=25.0.0')
libobsfrontend = cc.find_library('obs-frontend-api')

# Include paths
inc = include_directories(['/usr/include/obs'])

# Subdirectories
subdir('src')

