# ilsm - assemble_conf.py
# Licensed under the Apache 2.0 License

# Part of code of configurration of building custom linux bootc images

import os

# This is a example of ilsm project

#
#   project/
#   ├── rootfs/ <-- (optional) filesystem to be copied to image
#   ├── scripts/ <-- (optional) scripts to be executed after rootfs is copied
#   ├── prescripts/ <-- (optional) scripts to be executed before packages is installed
#   ├── ilsm.packages <-- list of packages to be installed
#   ├── ilsm.base <-- Base distro to be used for building the image (like debian, ubuntu, fedora, arch and opensuse)
#   └── ilsm.info <-- Information about the image


# Class to hold information about the image
class IlsmInfo:
    def __init__(self, name, version, description):
        self.name = name
        self.version = version
        self.description = description

class IlsmAssembleConfig:
    def __init__(self, base_distro, packages, rootfs=None, scripts=None, preScripts=None):
        self.base_distro = base_distro
        self.packages = packages
        self.rootfs = rootfs
        self.scripts = scripts
        self.preScripts = preScripts

# Search the folder there is the code runned and search for the configuration files and return the configuration object
def load_assemble_config():
    base_distro = None
    packages = []
    rootfs = None
    scripts = None
    preScripts = None

    # Load base distro from ilsm.base file
    if os.path.exists("ilsm.base"):
        with open("ilsm.base", "r") as f:
            base_distro = f.read().strip()

    # Load packages from ilsm.packages file
    if os.path.exists("ilsm.packages"):
        with open("ilsm.packages", "r") as f:
            packages = [line.strip() for line in f.readlines() if line.strip()]

    # Load rootfs from rootfs folder
    if os.path.exists("rootfs"):
        rootfs = "rootfs"

    # Load scripts from scripts folder
    if os.path.exists("scripts"):
        scripts = "scripts"

    # Load preScripts from preScripts folder
    if os.path.exists("prescripts"):
        preScripts = "prescripts"

    return IlsmAssembleConfig(base_distro, packages, rootfs, scripts, preScripts)