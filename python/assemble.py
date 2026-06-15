# ilsm - assembler.py
# Licensed under the Apache 2.0 License

# Part of code responsible of building custom linux bootc images

# Fedora is experimentally supported, but it is not fully tested yet, so it may not work as expected. It is recommended to use other distros for now.



CONTAINER_FILE_TEMPLATE = """

FROM scratch as ctx
COPY ../shared/ /shared

FROM {distro} as base

FROM base as system

RUN {package_manager_install} {packages}

{pacman_overlays}

{copy_kernel}

{clean_apt}

RUN --mount=type=tmpfs,dst=/tmp --mount=type=tmpfs,dst=/root \
    --mount=type=bind,from=ctx,source=/,target=/ctx \
    /ctx/shared/initramfs.sh

RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    echo "HOME=/var/home" | tee -a "/etc/default/useradd" && \
    /ctx/shared/bootc-rootfs.sh

{remove_usretc}

LABEL containers.bootc 1
LABEL ilsm.base {distro_name}

"""

# TODO: Add packages, rootfs, and postscripts
# Build a container file based on the detected distro and package manager

def tests():
    build_container_file("debian")
    build_container_file("ubuntu")
    build_container_file("fedora")
    build_container_file("arch")
    build_container_file("opensuse")
def build_container_file(distro):
    CONTAINERFILE = CONTAINER_FILE_TEMPLATE

    # NOTE: Copy kernel only in debian in ubuntu, because in other distros the kernel is in /usr/lib/modules
    if distro == "debian":
        CONTAINERFILE = CONTAINERFILE.format(distro="docker.io/library/debian:stable", 
        package_manager_install="apt-get update && apt-get install -y", 
        packages="", 
        copy_kernel="""RUN cp /boot/vmlinuz-* "$(find /usr/lib/modules -maxdepth 1 -type d | tail -n 1)/vmlinuz""", 
        pacman_overlays="", 
        clean_apt="RUN apt clean -y", 
        remove_usretc="",
        distro_name="debian")
    elif distro == "ubuntu":
        CONTAINERFILE = CONTAINERFILE.format(distro="docker.io/library/ubuntu:resolute", 
        package_manager_install="apt-get update && apt-get install -y", 
        packages="", 
        copy_kernel="""RUN cp /boot/vmlinuz-* "$(find /usr/lib/modules -maxdepth 1 -type d | tail -n 1)/vmlinuz""", 
        pacman_overlays="", 
        clean_apt="RUN apt clean -y", 
        remove_usretc="",
        distro_name="ubuntu")
    elif distro == "fedora":
        CONTAINERFILE = CONTAINERFILE.format(distro="docker.io/library/fedora:43",
        package_manager_install="dnf install -y --refresh",
        packages="",
        copy_kernel="",
        pacman_overlays="",
        clean_apt="",
        remove_usretc="",
        distro_name="fedora")
    elif distro == "arch":
        CONTAINERFILE = CONTAINERFILE.format(distro="docker.io/library/archlinux:latest",
        package_manager_install="pacman -Syu --noconfirm",
        packages="",
        copy_kernel="",
        pacman_overlays="""RUN grep "= */var" /etc/pacman.conf | sed "/= *\/var/s/.*=// ; s/ //" | xargs -n1 sh -c 'mkdir -p "/usr/lib/sysimage/$(dirname $(echo $1 | sed "s@/var/@@"))" && mv -v "$1" "/usr/lib/sysimage/$(echo "$1" | sed "s@/var/@@")"' '' && \
    sed -i -e "/= *\/var/ s/^#//" -e "s@= */var@= /usr/lib/sysimage@g" -e "/DownloadUser/d" /etc/pacman.conf""",
        clean_apt="",
        remove_usretc="",
        distro_name="arch")

    elif distro == "opensuse":
        CONTAINERFILE = CONTAINERFILE.format(distro="registry.opensuse.org/opensuse/tumbleweed:latest",
        package_manager_install="zypper refresh && zypper install -y",
        packages="",
        copy_kernel="",
        pacman_overlays="",
        clean_apt="",
        remove_usretc="RUN rm -rf /usr/etc",
        distro_name="opensuse")

    print(CONTAINERFILE)




