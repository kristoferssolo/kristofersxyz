#!/bin/sh
# Kristofers Auto Rice Boostrapping Script (KARBS)
# by Kristofers Solo
# License: GNU GPLv3

echo "Choose display server (X11 / Wayland)[1/2]: "
read -r USER_INPUT


# Get display server type from user
if [ "$USER_INPUT" = 1 ]; then
    DISPLAY_SERVER="X11"
elif [ "$USER_INPUT" = 2 ]; then
    DISPLAY_SERVER="wayland"
else
    echo "Wrong input. Please try again."
    exit
fi


# Install paru
if pacman -Q paru; then
    :
else
    sudo pacman -S --noconfirm rust-src git wget
    git clone https://aur.archlinux.org/paru-bin
    cd paru-bin || exit
    makepkg -si
    cd ..
    rm -rf paru-bin
fi

FILE="pkg-files/$DISPLAY_SERVER-pkgs"

if [ -f "$FILE" ]; then
    paru -Syu --noconfirm --needed - < $FILE
else
    curl -LO https://raw.githubusercontent.com/kristoferssolo/karbs/main/$FILE
    paru -Syu --noconfirm --needed - < $DISPLAY_SERVER-pkgs
    rm -f $DISPLAY_SERVER-pkgs
fi

git clone https://github.com/kristoferssolo/solorice "$HOME"
git clone https://github.com/kristoferssolo/SoloVim "$HOME"/.config/nvim
rm -rf "$HOME"/{LICENSE,readme.md,.gitignore}
mkdir -p "$HOME"/{Downloads,Documents,Videos,Music,Pictures/screenshots}

if [ $DISPLAY_SERVER = "wayland" ]; then
    rm -rf "$HOME"/.config/{awesome,picom,sx,zsh/.zprofile-X11}
    mv "$HOME"/.config/zsh/zprofile-wayland "$HOME"/.config/zsh/.zprofile
    chsh -s /bin/zsh
    zsh
    Hyprland
else
    rm -rf "$HOME"/.config/{hypr,waybar,zsh/.zprofile-wayland}
    mv "$HOME"/.config/zsh/zprofile-X11 "$HOME"/.config/zsh/.zprofile
    git clone https://github.com/streetturtle/awesome-wm-widgets "$HOME"/.config/awesome/awesome-wm-widgets
    chsh -s /bin/zsh
    zsh
    echo -e "\n\n\033[1;31mFor weather widget to work, enter API-key from https://openweathermap.org, latitude and longitude in '~/.config/awesome/weather' file, each on seperate line.\033[0m"
    echo "API-key"
    echo "latitude"
    echo "longitude"
    echo -e "\nEverything else is ready to go. You can run 'sx' or reboot."
fi
