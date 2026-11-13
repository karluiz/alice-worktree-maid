#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"

cup_shape='polygon 550,220 620,220 625,255 600,270 565,260 550,245'
leg_shape='polygon 500,400 570,405 600,500 580,550 530,550 500,480'
arrow_shape='polygon 690,480 735,480 740,580 690,580'
cup_dy=(0 -6 -16 -12 -2 6 2 0)
leg_angles=(0 -4 -9 -4 4 9 4 0)
arrow_dy=(0 -4 -10 -6 2 8 3 0)

for theme in light dark; do
    if [[ "$theme" == light ]]; then
        source='rest.jpg'
        prefix='rest-rigid'
    else
        source='rest-dark.png'
        prefix='rest-rigid-dark'
        magick "$source" -background black -flatten "${prefix}-source.png"
        source="${prefix}-source.png"
    fi

    for part in cup leg arrow; do
        shape_var="${part}_shape"
        magick -size 1200x896 xc:black -fill white -draw "${!shape_var}" "${prefix}-${part}-mask.png"
        magick "$source" "${prefix}-${part}-mask.png" -alpha off -compose CopyOpacity -composite "${prefix}-${part}.png"
    done

    if [[ "$theme" == light ]]; then
        magick "$source" -fill white -draw "$cup_shape" -draw "$leg_shape" -draw "$arrow_shape" "${prefix}-base.png"
    else
        for part in cup leg arrow; do
            magick "${prefix}-${part}-mask.png" -negate "${prefix}-${part}-remove.png"
        done
        magick "$source" "${prefix}-cup-remove.png" -alpha off -compose CopyOpacity -composite "${prefix}-base-0.png"
        magick "${prefix}-base-0.png" "${prefix}-leg-remove.png" -alpha off -compose CopyOpacity -composite "${prefix}-base-1.png"
        magick "${prefix}-base-1.png" "${prefix}-arrow-remove.png" -alpha off -compose CopyOpacity -composite -background black -alpha remove "${prefix}-base.png"
    fi

    magick "$source" "${prefix}-0.png"
    for index in 1 2 3 4 5 6 7; do
        magick "${prefix}-base.png" \
            \( "${prefix}-cup.png" -roll "+0+${cup_dy[$index]}" \) -compose Over -composite \
            \( "${prefix}-leg.png" -distort SRT "560,410 ${leg_angles[$index]}" \) -compose Over -composite \
            \( "${prefix}-arrow.png" -roll "+0+${arrow_dy[$index]}" \) -compose Over -composite \
            "${prefix}-${index}.png"
    done

    if [[ "$theme" == light ]]; then
        magick "${prefix}-5.png" -fill white -draw 'rectangle 590,180 620,210' -draw 'rectangle 648,180 678,210' -fill none -stroke black -strokewidth 3 -draw 'line 593,196 617,196' -draw 'line 651,196 675,196' "${prefix}-5.png"
    else
        magick -size 1200x896 xc:black -fill white -draw 'rectangle 590,180 620,210' -draw 'rectangle 648,180 678,210' -negate "${prefix}-blink-mask.png"
        magick "${prefix}-5.png" "${prefix}-blink-mask.png" -alpha off -compose CopyOpacity -composite -background black -alpha remove -fill none -stroke white -strokewidth 3 -draw 'line 593,196 617,196' -draw 'line 651,196 675,196' "${prefix}-5.png"
    fi
done

for prefix in rest-rigid rest-rigid-dark; do
    rm -f "${prefix}-cup-mask.png" "${prefix}-cup.png" "${prefix}-cup-remove.png" \
        "${prefix}-leg-mask.png" "${prefix}-leg.png" "${prefix}-leg-remove.png" \
        "${prefix}-arrow-mask.png" "${prefix}-arrow.png" "${prefix}-arrow-remove.png" \
        "${prefix}-base"*.png "${prefix}-source.png" "${prefix}-blink-mask.png"
done
