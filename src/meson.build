obs_controller_src = [
  'main.cpp',
  'api.cpp'
]

obs_controller_deps = [
  libobs,
  libobsfrontend
]


libobscontroller = library('obs-controller', obs_controller_src, dependencies: obs_controller_deps, include_directories: inc)
libobscontroller_dep = declare_dependency(dependencies:        obs_controller_deps,
                                          include_directories: [include_directories('.')],
                                          link_with:           libobscontroller)

