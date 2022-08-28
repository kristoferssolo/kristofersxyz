#!/bin/sh
echo 'Choose installation size: minimal or full'
read size

if pacman -Q paru; then
    echo
else
    sudo pacman -S --noconfirm rust
    git clone 'https://aur.archlinux.org/paru-bin'
    cd paru-bin
    makepkg -si
    cd ..
    rm -rf paru-bin
fi

FILE = "pkg-files/$size-pkgs.txt"

if [[ -f "$FILE" ]]; then
    paru -Syu --noconfirm --needed - < "pkg-files/$size-pkgs"
else
    curl -LO "https://raw.githubusercontent.com/kristoferssolo/karbs/main/pkg-files/$size-pkgs"
    paru -Syu --noconfirm --needed - < "$size-pkgs"
    rm "$size"-pkgs
fi

mkdir -p "$HOME"/{repos,Downloads,Documents,Videos,Music,Pictures/screenshots}
git clone 'https://github.com/kristoferssolo/solorice' "$HOME/repos/solorice"

cp -rf "$HOME/repos/solorice/.config" "$HOME"
rm -rf "$HOME/.config/awesome/desktop"
touch "$HOME/.config/awesome/weather"
cp -rf "$HOME/repos/solorice/.local" "$HOME"
ln -sf "$HOME/.config/zsh/.zshenv" "$HOME"
git clone 'https://github.com/streetturtle/awesome-wm-widgets' "$HOME/.config/awesome/awesome-wm-widgets"

chsh -s /bin/zsh

echo
echo
echo -e '\033[1;31m For weather widget to work, enter API-key from https://openweathermap.org, latitude and logitude in `~/.config/awesome/weather` file, each on seperate line. \033[0m'
