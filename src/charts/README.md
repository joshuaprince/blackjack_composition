# How to create a Basic Strategy chart .CSV file

1. Go to https://wizardofodds.com/games/blackjack/strategy/calculator/
2. Input desired rules
3. Copy the entire outputted table as-is, starting at the word "Hard" and ending at the last directive in the 
   bottom-right corner
4. Paste into a file
5. "Replace" all tab characters `\t` with commas `,`
6. "Replace" remaining space characters with nothing
7. Under the "Pair" section, remove all duplicate numbers - e.g. "2,2,P,P..." should become "2,P,P..."
8. Add a row for "Hard 4" by duplicating the "Hard 5" row
9. Add a row for "Soft 12" by duplicating the "Soft 13" row 