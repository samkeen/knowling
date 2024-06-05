/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./src/**/*.{vue,js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            typography: (theme) => ({
                DEFAULT: {
                    css: {
                        maxWidth: 'none',  // override here
                    },
                },
            }),
        },
    },
    plugins: [
        require('@tailwindcss/typography'),
        require("daisyui"),
    ],
    daisyui: {
        themes: [{
            "light": {
                ...require("daisyui/src/theming/themes")["light"],
                info: "#efefef"
            },
            "dark": {
                ...require("daisyui/src/theming/themes")["dark"],
            }
        }],
    },
}

