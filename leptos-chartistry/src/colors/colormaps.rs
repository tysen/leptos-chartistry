// The Scientific color maps are licensed under a MIT License
//
// Copyright (c) 2023, Fabio Crameri
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// The Scientific color maps by Fabio Crameri (Crameri 2018) prevent visual distortion of the data and exclusion of readers with color­-vision deficiencies (Crameri et al., 2020) and the overview graphic is available via the open-access s-ink.org repository.

/*
Notes:

Colors are an important part of charts. Our aim is to avoid less readable and misleading color schemes. So we rely on the scientific color maps developed by Fabio Crameri. These are perceptually uniform, color blind friendly, and monochrome friendly.

Reading material:
- Summary poster: https://www.fabiocrameri.ch/ws/media-library/a17d02961b3a4544961416de2d7900a4/posterscientificcolormaps_crameri.pdf
- Article "The misuse of color in science communication" https://www.nature.com/articles/s41467-020-19160-7
- Homepage https://www.fabiocrameri.ch/colormaps/
- Flow chart on picking a scheme: https://s-ink.org/color-map-guideline
- Available color schemes: https://s-ink.org/scientific-color-maps
*/

use super::{scheme::DivergingGradient, Color, SequentialGradient};

pub const BATLOW: [Color; 10] = [
    Color::from_rgb(0x01, 0x19, 0x59),
    Color::from_rgb(0x10, 0x3F, 0x60),
    Color::from_rgb(0x1C, 0x5A, 0x62),
    Color::from_rgb(0x3C, 0x6D, 0x56),
    Color::from_rgb(0x68, 0x7B, 0x3E),
    Color::from_rgb(0x9D, 0x89, 0x2B),
    Color::from_rgb(0xD2, 0x93, 0x43),
    Color::from_rgb(0xF8, 0xA1, 0x7B),
    Color::from_rgb(0xFD, 0xB7, 0xBC),
    Color::from_rgb(0xFA, 0xCC, 0xFA),
];

pub const LIPARI: SequentialGradient = (
    Color::from_rgb(0x03, 0x13, 0x26),
    &[
        Color::from_rgb(0x13, 0x38, 0x5A),
        Color::from_rgb(0x47, 0x58, 0x7A),
        Color::from_rgb(0x6B, 0x5F, 0x76),
        Color::from_rgb(0x8E, 0x61, 0x6C),
        Color::from_rgb(0xBC, 0x64, 0x61),
        Color::from_rgb(0xE5, 0x7B, 0x62),
        Color::from_rgb(0xE7, 0xA2, 0x79),
        Color::from_rgb(0xE9, 0xC9, 0x9F),
        Color::from_rgb(0xFD, 0xF5, 0xDA),
    ],
);

pub const BERLIN: DivergingGradient = (
    (
        Color::from_rgb(0x9E, 0xB0, 0xFF),
        &[
            Color::from_rgb(0x5B, 0xA4, 0xDB),
            Color::from_rgb(0x2D, 0x75, 0x97),
            Color::from_rgb(0x1A, 0x42, 0x56),
            Color::from_rgb(0x11, 0x19, 0x1E),
        ],
    ),
    (
        Color::from_rgb(0x28, 0x0D, 0x01),
        &[
            Color::from_rgb(0x50, 0x18, 0x03),
            Color::from_rgb(0x8A, 0x3F, 0x2A),
            Color::from_rgb(0xC4, 0x75, 0x6A),
            Color::from_rgb(0xFF, 0xAD, 0xAD),
        ],
    ),
);
