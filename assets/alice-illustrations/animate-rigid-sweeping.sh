#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"

broom_shape='polygon 879,408 891,419 778,566 797,575 789,594 770,610 744,620 711,612 693,598 697,580 710,585 728,583 745,571 858,421 864,408'
debris_a='polygon 836,520 862,525 885,565 872,590 845,580 825,555'
debris_b='polygon 810,570 840,575 850,610 830,625 805,615 800,595'
debris_c='polygon 865,575 895,575 905,610 885,630 860,620'
angles=(0 -10 -18 -8 8 18 8 0)
dx=(0 -10 -28 -14 12 28 12 0)
dy=(0 -10 -26 -34 -20 -12 -4 0)

for theme in light dark; do
    if [[ "$theme" == light ]]; then
        source='sweeping.jpg'
        prefix='sweeping-rigid'
    else
        source='sweeping-dark.png'
        prefix='sweeping-rigid-dark'
        magick "$source" -background black -flatten "${prefix}-source.png"
        source="${prefix}-source.png"
    fi

    magick -size 1376x768 xc:black -fill white -draw "$broom_shape" "${prefix}-broom-mask.png"
    magick "$source" "${prefix}-broom-mask.png" -alpha off -compose CopyOpacity -composite "${prefix}-broom.png"

    for part in a b c; do
        shape_var="debris_${part}"
        magick -size 1376x768 xc:black -fill white -draw "${!shape_var}" "${prefix}-${part}-mask.png"
        magick "$source" "${prefix}-${part}-mask.png" -alpha off -compose CopyOpacity -composite "${prefix}-${part}.png"
    done

    if [[ "$theme" == light ]]; then
        magick "$source" -fill white -draw "$broom_shape" -draw "$debris_a" -draw "$debris_b" -draw "$debris_c" "${prefix}-base.png"
    else
        for part in broom a b c; do
            magick "${prefix}-${part}-mask.png" -negate "${prefix}-${part}-remove.png"
        done
        magick "$source" "${prefix}-broom-remove.png" -alpha off -compose CopyOpacity -composite "${prefix}-base-0.png"
        magick "${prefix}-base-0.png" "${prefix}-a-remove.png" -alpha off -compose CopyOpacity -composite "${prefix}-base-1.png"
        magick "${prefix}-base-1.png" "${prefix}-b-remove.png" -alpha off -compose CopyOpacity -composite "${prefix}-base-2.png"
        magick "${prefix}-base-2.png" "${prefix}-c-remove.png" -alpha off -compose CopyOpacity -composite -background black -alpha remove "${prefix}-base.png"
    fi

    magick "$source" "${prefix}-0.png"
    for index in 1 2 3 4 5 6 7; do
        magick "${prefix}-base.png" \
            \( "${prefix}-broom.png" -distort SRT "875,420 ${angles[$index]}" \) -compose Over -composite \
            \( "${prefix}-a.png" -roll "+${dx[$index]}+${dy[$index]}" \) -compose Over -composite \
            \( "${prefix}-b.png" -roll "+${dx[$index]}+${dy[$index]}" \) -compose Over -composite \
            \( "${prefix}-c.png" -roll "+${dx[$index]}+${dy[$index]}" \) -compose Over -composite \
            "${prefix}-${index}.png"
    done

    if [[ "$theme" == light ]]; then
        magick "${prefix}-5.png" -fill white -draw 'rectangle 888,264 906,292' -draw 'rectangle 936,260 954,288' -fill none -stroke black -strokewidth 3 -draw 'line 889,280 905,280' -draw 'line 937,278 953,278' "${prefix}-5.png"
    else
        magick -size 1376x768 xc:black -fill white -draw 'rectangle 888,264 906,292' -draw 'rectangle 936,260 954,288' -negate "${prefix}-blink-mask.png"
        magick "${prefix}-5.png" "${prefix}-blink-mask.png" -alpha off -compose CopyOpacity -composite -background black -alpha remove -fill none -stroke white -strokewidth 3 -draw 'line 889,280 905,280' -draw 'line 937,278 953,278' "${prefix}-5.png"
    fi
done

for prefix in sweeping-rigid sweeping-rigid-dark; do
    rm -f "${prefix}-broom-mask.png" "${prefix}-broom.png" "${prefix}-broom-clean.png" \
        "${prefix}-a-mask.png" "${prefix}-a.png" "${prefix}-a-remove.png" \
        "${prefix}-b-mask.png" "${prefix}-b.png" "${prefix}-b-remove.png" \
        "${prefix}-c-mask.png" "${prefix}-c.png" "${prefix}-c-remove.png" \
        "${prefix}-broom-remove.png" "${prefix}-base"*.png "${prefix}-source.png" \
        "${prefix}-blink-mask.png"
done
