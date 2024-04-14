/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./src/**/*.{vue,js,ts,jsx,tsx}",
        // use of prose class comes from Milkdown, thus you need to
        // have postcss search there in case you are not useing prose
        // class in you HTML
        "./node_modules/@milkdown/**/*.{vue,js,ts,jsx,tsx}",
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
        themes: ["light", "dark", "cupcake"],
    },
}

