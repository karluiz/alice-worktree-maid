#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"

hand_shape='polygon 590,400 640,430 640,480 590,500 570,470'
plug_shape='polygon 785,495 845,500 850,575 820,625 770,605 760,550'
plant_shape='polygon 335,520 410,520 430,640 420,700 340,700 320,630'
hand_angles=(0 -4 -10 -5 3 9 4 0)
plug_dy=(0 -4 -10 -6 2 6 2 0)
plant_angles=(0 -2 -4 -2 2 4 2 0)

for theme in light dark; do
    if [[ "$theme" == light ]]; then
        source='greeting.jpg'
        prefix='greeting-rigid'
    else
        source='greeting-dark.png'
        prefix='greeting-rigid-dark'
        magick "$source" -background black -flatten "${prefix}-source.png"
        source="${prefix}-source.png"
    fi

    for part in hand plug plant; do
        shape_var="${part}_shape"
        magick -size 1200x896 xc:black -fill white -draw "${!shape_var}" "${prefix}-${part}-mask.png"
        magick "$source" "${prefix}-${part}-mask.png" -alpha off -compose CopyOpacity -composite "${prefix}-${part}.png"
    done

    if [[ "$theme" == light ]]; then
        magick "$source" -fill white -draw "$hand_shape" -draw "$plug_shape" -draw "$plant_shape" "${prefix}-base.png"
    else
        for part in hand plug plant; do
            magick "${prefix}-${part}-mask.png" -negate "${prefix}-${part}-remove.png"
        done
        magick "$source" "${prefix}-hand-remove.png" -alpha off -compose CopyOpacity -composite "${prefix}-base-0.png"
        magick "${prefix}-base-0.png" "${prefix}-plug-remove.png" -alpha off -compose CopyOpacity -composite "${prefix}-base-1.png"
        magick "${prefix}-base-1.png" "${prefix}-plant-remove.png" -alpha off -compose CopyOpacity -composite -background black -alpha remove "${prefix}-base.png"
    fi

    magick "$source" "${prefix}-0.png"
    for index in 1 2 3 4 5 6 7; do
        magick "${prefix}-base.png" \
            \( "${prefix}-hand.png" -distort SRT "610,420 ${hand_angles[$index]}" \) -compose Over -composite \
            \( "${prefix}-plug.png" -roll "+0+${plug_dy[$index]}" \) -compose Over -composite \
            \( "${prefix}-plant.png" -distort SRT "380,670 ${plant_angles[$index]}" \) -compose Over -composite \
            "${prefix}-${index}.png"
    done

    if [[ "$theme" == light ]]; then
        magick "${prefix}-2.png" -fill white -draw 'rectangle 508,320 532,355' -draw 'rectangle 552,320 578,355' -fill none -stroke black -strokewidth 3 -draw 'line 510,340 530,340' -draw 'line 554,340 574,340' "${prefix}-2.png"
    else
        magick -size 1200x896 xc:black -fill white -draw 'rectangle 508,320 532,355' -draw 'rectangle 552,320 578,355' -negate "${prefix}-blink-mask.png"
        magick "${prefix}-2.png" "${prefix}-blink-mask.png" -alpha off -compose CopyOpacity -composite -background black -alpha remove -fill none -stroke white -strokewidth 3 -draw 'line 510,340 530,340' -draw 'line 554,340 574,340' "${prefix}-2.png"
    fi
done

for prefix in greeting-rigid greeting-rigid-dark; do
    rm -f "${prefix}-hand-mask.png" "${prefix}-hand.png" "${prefix}-hand-remove.png" \
        "${prefix}-plug-mask.png" "${prefix}-plug.png" "${prefix}-plug-remove.png" \
        "${prefix}-plant-mask.png" "${prefix}-plant.png" "${prefix}-plant-remove.png" \
        "${prefix}-base"*.png "${prefix}-source.png" "${prefix}-blink-mask.png"
done
