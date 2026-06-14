# ilsm - detector.py
# Licensed under the Apache 2.0 License

# Part of code responsible of detecting linux distro in os-release file


# Reads the os-release file and returns the value of ID or ID_LIKE field, which is the distro name
# Supported distros: debian, ubuntu, fedora, arch, opensuse

def detect_distro(os_release_path):
    supported_distros = ["debian", "ubuntu", "fedora", "arch", "opensuse"]

    distro_id = None
    distro_id_like = None

    with open(os_release_path, "r") as f:
        for line in f:
            if line.startswith("ID="):
                distro_id = line.strip().split("=")[1].strip('"')
            elif line.startswith("ID_LIKE="):
                distro_id_like = line.strip().split("=")[1].strip('"')

    if distro_id in supported_distros:
        return distro_id

    if distro_id_like:
        for like in distro_id_like.split():
            if like in supported_distros:
                return like

    return None


def detect_package_manager(distro):
    if distro in ["debian", "ubuntu"]:
        return "apt"
    elif distro == "fedora":
        return "dnf"
    elif distro == "arch":
        return "pacman"
    elif distro == "opensuse":
        return "zypper"
    else:
        return None