# ilsm - assemble.py
# Licensed under the Apache 2.0 License

# Part of code responsible of building custom linux bootc images

# Fedora is experimentally supported, but it is not fully tested yet, so it may not work as expected. It is recommended to use other distros for now.

import os
import assemble_conf
import shutil

CONTAINER_FILE_TEMPLATE = """

FROM scratch as ctx
COPY shared/ /shared/bootc-scripts
COPY scripts/ /shared/scripts
COPY prescripts/ /shared/prescripts

FROM scratch as rootfs
COPY rootfs/ /

FROM {distro} as base

FROM base as system

# Execute all prescripts in the /shared directory

RUN --mount=type=tmpfs,dst=/tmp --mount=type=tmpfs,dst=/root \
    --mount=type=bind,from=ctx,source=/,target=/ctx \
    find /ctx/shared/prescripts -type f -name '*.sh' -exec sh {} \;

{pacman_overlays}

RUN {package_manager_install} {packages}

# Copy user rootfs
COPY --from=rootfs / /

{copy_kernel}

{clean_apt}

RUN --mount=type=tmpfs,dst=/tmp --mount=type=tmpfs,dst=/root \
    --mount=type=bind,from=ctx,source=/,target=/ctx \
    /ctx/shared/bootc-scripts/initramfs.sh

RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    echo "HOME=/var/home" | tee -a "/etc/default/useradd" && \
    /ctx/shared/bootc-scripts/bootc-rootfs.sh

# Execute all postscripts
RUN --mount=type=tmpfs,dst=/tmp --mount=type=tmpfs,dst=/root \
    --mount=type=bind,from=ctx,source=/,target=/ctx \
    find /ctx/shared/scripts -type f -name '*.sh' -exec sh {} \;

{remove_usretc}

LABEL containers.bootc 1
LABEL ilsm.base {distro_name}

"""

# TODO: Add packages, rootfs, and postscripts
# Build a container file based on the detected distro and package manager


def assemble_image():
    # Create tmp dir for building the image
    os.makedirs("/tmp/ilsm/assemble", exist_ok=True)

    assembleConf = assemble_conf.load_assemble_config()

    if not assembleConf.base_distro:
        raise ValueError("ilsm.base was not found!")

    # Copy scripts, rootfs, and prescripts to /tmp/ilsm/assemble
    if assembleConf.rootfs:
        shutil.copytree(
            assembleConf.rootfs,
            "/tmp/ilsm/assemble/rootfs",
            dirs_exists_ok=True
        )

    if assembleConf.preScripts:
        shutil.copytree(
            assembleConf.preScripts,
            "/tmp/ilsm/assemble/prescripts",
            dirs_exists_ok=True
        )

    if assembleConf.scripts:
        shutil.copytree(
            assembleConf.scripts,
            "/tmp/ilsm/assemble/scripts",
            dirs_exists_ok=True
        )

    distro = assembleConf.base_distro
    containerFile = build_container_file(distro, assembleConf)
    with open(path, "w"):
        f.write(containerFile)

def build_container_file(distro, conf):
    CONTAINERFILE = CONTAINER_FILE_TEMPLATE

    # NOTE: Copy kernel only in debian in ubuntu, because in other distros the kernel is in /usr/lib/modules
    if distro == "debian":
        CONTAINERFILE = CONTAINERFILE.format(distro="docker.io/library/debian:stable", 
        package_manager_install="apt-get update && apt-get install -y", 
        packages="".join(conf.packages), 
        copy_kernel="""RUN cp /boot/vmlinuz-* "$(find /usr/lib/modules -maxdepth 1 -type d | tail -n 1)/vmlinuz""", 
        pacman_overlays="", 
        clean_apt="RUN apt clean -y", 
        remove_usretc="",
        distro_name="debian")
    elif distro == "ubuntu":
        CONTAINERFILE = CONTAINERFILE.format(distro="docker.io/library/ubuntu:resolute", 
        package_manager_install="apt-get update && apt-get install -y", 
        packages="".join(conf.packages), 
        copy_kernel="""RUN cp /boot/vmlinuz-* "$(find /usr/lib/modules -maxdepth 1 -type d | tail -n 1)/vmlinuz""", 
        pacman_overlays="", 
        clean_apt="RUN apt clean -y", 
        remove_usretc="",
        distro_name="ubuntu")
    elif distro == "fedora":
        CONTAINERFILE = CONTAINERFILE.format(distro="docker.io/library/fedora:43",
        package_manager_install="dnf install -y --refresh",
        packages="".join(conf.packages),
        copy_kernel="",
        pacman_overlays="",
        clean_apt="",
        remove_usretc="",
        distro_name="fedora")
    elif distro == "arch":
        CONTAINERFILE = CONTAINERFILE.format(distro="docker.io/library/archlinux:latest",
        package_manager_install="pacman -Syu --noconfirm",
        packages="".join(conf.packages),
        copy_kernel="",
        pacman_overlays="""RUN grep "= */var" /etc/pacman.conf | sed "/= *\/var/s/.*=// ; s/ //" | xargs -n1 sh -c 'mkdir -p "/usr/lib/sysimage/$(dirname $(echo $1 | sed "s@/var/@@"))" && mv -v "$1" "/usr/lib/sysimage/$(echo "$1" | sed "s@/var/@@")"' '' && \
    sed -i -e "/= *\/var/ s/^#//" -e "s@= */var@= /usr/lib/sysimage@g" -e "/DownloadUser/d" /etc/pacman.conf""",
        clean_apt="",
        remove_usretc="",
        distro_name="arch")

    elif distro == "opensuse":
        CONTAINERFILE = CONTAINERFILE.format(distro="registry.opensuse.org/opensuse/tumbleweed:latest",
        package_manager_install="zypper refresh && zypper install -y",
        packages="".join(conf.packages),
        copy_kernel="",
        pacman_overlays="",
        clean_apt="",
        remove_usretc="RUN rm -rf /usr/etc",
        distro_name="opensuse")

    else:
        raise ValueError(f"Unknown distro: {distro}")

    return CONTAINERFILE




