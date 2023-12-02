# Rust Blackjack Composition Analyzer and Simulator

This project started with a few of my own curiosities:

1. "Advantage Players" in casino Blackjack know how
   to [deviate](https://www.blackjackapprenticeship.com/blackjack-deviations/) from a basic strategy based on the
   frequency of remaining cards in the deck. If a computer were to perfectly analyze the remaining deck composition in a
   fair game of Blackjack, how frequently would its playing choices deviate from basic strategy?
2. How much of an advantage would that computer have compared to playing perfect basic strategy even if it never changed
   its bets like an Advantage Player would?
3. What's the big deal with [Rust](https://www.rust-lang.org/)?

To answer these, I built a program that simulates as many games of Blackjack as my 24-thread CPU could handle. For every
decision the player makes:

- Calculate the optimal decision to hit, stand, double, or split. Since every decision comes with a unique set of cards
  in play and remaining cards in the deck, I **brute-force the exact odds of every outcome** to be as precise as
  possible.
- Compare the expected value of that choice with the expected value of the action suggested by a basic strategy chart.

Rust was the perfect choice for the job since this brute-force simulation is CPU-intensive, and I get plenty of manual
memory management at my day job.

<div align="center">
<img src="./.github/chatgpt_generated_robot.webp" alt="An AI-generated image of a robot playing Blackjack" height="400px"/>
<br>
<i>Image courtesy of ChatGPT 4.</i>
</div>

## The Setup

The casino's edge in Blackjack is already very slim with standard rules. A proper game will carry an edge of around
**0.5%**, which means that for every $100 wagered on the game, the player can expect to lose around 50 cents in the long
run if played according to basic strategy.

I chose rules for my simulation based on the most common rules I've seen for single-deck games: 1 deck, shuffled after
half of the cards have been played, that pays 3:2 on a player Blackjack. The dealer hits a soft 17. The player can split
up to 4 hands, but Aces can only be split once and can only take one additional card. Most restrictively, the player can
only double down a hard 10 or 11, and cannot double down after a split. These rules can all be [tweaked](./src/rules.rs)
in the simulation.

Notably, I only chose to simulate single-deck. The goal is to determine an upper bound of "how much better can a
computer play", and even two decks severely dilute the amount of information available to the computer on any given
hand.

## The Results

I let the simulation run overnight a few times. Full records are logged in [record.txt](./record.txt). Here's an
example:

```
Played 144180610 hands (27931824 shoes) and had total of -70705.5 returned. Edge = -0.04903953451161013%,
2261 hands/sec total (2238 hands/433 shoes in last second), 11846522/197323361 deviant actions
0.44857870762458846% average +EV/hand. 872283/2368688/11058450 insurances won/taken/offered (+0.08587506541799021 EV).
```

<details open>
<summary>Click for full deviations breakdown by hand.</summary>
<br>

Left axis is the player's hand. Top axis is the dealer's up-card. In each cell, the left number is the number of times
the computer chose to deviate from basic strategy in that situation. The right number is the total number of times that
situation came up.

```
Hard         2                 3                 4                 5                 6                 7                 8                 9                10                 A
4           0/0               0/0               0/0               0/0               0/0               0/0               0/0               0/0               0/0               0/0       4
5           0/111312          0/114802          0/159739          0/160048          0/160198          0/156548          0/145393          0/141915          0/518413          0/95510   5
6           0/108750          0/150011          0/113471          0/152190          0/152695          0/148971          0/144431          0/143489          0/525938          0/96827   6
7           0/258803          0/266588          0/273562          0/273436          0/312062          0/306646          0/298868          0/295116          0/1080338         0/198721  7
8           0/272629          0/281258          0/326026          0/290986          0/287577          0/322272          0/312109          0/308114          0/1126698         0/206845  8
9           0/444534          0/453161          0/463356          0/471506          0/468020          0/456335          0/482023          0/474659          0/1730881         0/319016  9
10       5259/474108       4033/482118       2571/494588       1342/532661        941/499300      17345/490565      35270/475757      85410/506620     330501/1847260     74663/338871  10
11       4932/655005       4200/666773       2716/676796       1502/682471       1261/682496      22736/670014      43681/662261      67863/653544     330955/2494574     90922/462466  11
12     292039/1062909    468358/1187625    553490/1211680    305156/1222185    247231/1267248         0/1189847      1312/1171718     19231/1183683     81524/4408908      7761/828961  12
13     433669/1298407    309049/1167930    207115/1288401    138814/1282682    100939/1271007      4740/1317249      1835/1302444     22143/1310392    277934/4871807     32977/896795  13
14     211004/1160388    137785/1153072     79196/1009844     54722/1098861     38157/1098202     46688/1259945      8843/1242948     25714/1262201    466692/4700130     60843/867841  14
15      90245/1209187     64299/1191650     39358/1187633     25971/1039177     19660/1127515     86657/1363450     67550/1364435    148459/1382570    913009/5095014    115137/947851  15
16      44614/1081997     33311/1075119     20580/1041977     14431/1031039     20028/874675     185822/1300625    163655/1348995    300101/1316832   1970236/4835503    185790/909275  16
17        783/1107663       652/1095327       390/1074329      1155/1054797       870/1046895      3671/1321973    156571/1421617     49108/1411174     42610/5109052    142943/980830  17
18          0/892363          0/874875          0/858693          0/831315          0/819997          0/1248231         0/1119866        32/1305915         0/4549721        20/905471  18
19          0/888345          0/872126          0/855775          0/836141          0/825143          0/1196847         0/1280428         0/1099982         0/4259114         0/877481  19
20          0/177800          0/159819          0/134663          0/109254          0/100981          0/509054          0/536790          0/551233          0/2203217         0/444481  20
21          0/180406          0/158261          0/129857          0/101154          0/90553           0/524128          0/547780          0/533006          0/1930677         0/403401  21
Soft         2                 3                 4                 5                 6                 7                 8                 9                10                 A
12          0/0               0/0               0/0               0/0               0/0               0/0               0/0               0/0               0/0               0/0       12
13          0/108144          0/148718          0/149261          0/149415          0/149991          0/148248          0/141168          0/140452          0/526750          0/70948   13
14          0/150806          0/117137          0/158669          0/158571          0/159112          0/156340          0/151799          0/149247          0/557602          0/73670   14
15          0/156012          0/158654          0/124827          0/160184          0/160643          0/159079          0/161043          0/160657          0/605673          0/80080   15
16          0/177720          0/178166          0/186526          0/153277          0/188107          0/186364          0/185564          0/184086          0/690880          0/91242   16
17        119/197827          7/202232        111/198647        200/205603        289/167928       4991/202125          1/200223          1/199702         13/750097          0/98920   17
18       9427/226134       6930/228194       6331/229245       5376/230297      18939/237131         68/198223      15644/229929       8052/226189     105882/847710      12703/111502  18
19          0/233244          5/235065          9/238315          0/237037          0/236188          0/244240          0/203931          1/253445       6163/945014          2/123649  19
20          0/248661          0/245984          0/249644          0/251073          0/251990          0/243218          0/256014          0/232761          0/977960          0/133037  20
21          0/73659           0/79246           0/85257           0/96930           0/96988           0/75022           0/75116           0/97424           0/439669          0/57439   21
Pair         2                 3                 4                 5                 6                 7                 8                 9                10                 A
2        8482/27496       16317/57088        9956/57723        4013/57829        3102/58088       11376/56557        7658/53753         906/53414        1008/193890         66/36202   2
3       14619/54921       14296/27436        4339/57630        2103/57572        2883/57485       10561/56416       17390/54352        4910/53164        6324/194906        808/35781   3
4         151/53825         556/53348        1081/26648        8356/53693       12211/54015         248/52568         316/52940         114/52809         180/191925          9/35808   4
5         562/53341         370/52971         223/52998          35/26538          61/52566        1558/52842        3804/52542        8686/52293       42275/192506       9598/35282   5
6       15764/56446        9086/56337        3528/56603        1397/56919         506/27315       23708/54218        9588/53095        2926/52505        6050/191664       1126/35152   6
7        1675/57156        1054/56995         539/56630         218/56960          80/56618           4/27369       19807/52907        6075/52315       79407/190222       7639/35301   7
8           1/56951           0/57105           0/56736           6/56596          20/56881           0/56430           0/26942        7532/55953       36658/201911      10361/36986   8
9       12077/55969       10889/55601        7275/55837        3469/55988        3057/56314        5548/52289        2350/56078        1146/26808        1841/188999      13995/35843   9
10      17100/1051650     40591/1064379     81490/1086505    133077/1111364    138603/1116528      6297/1035039        89/1029499         0/1027439         0/3293996        23/737914  0
A         459/52455         346/51981         247/51868         191/52411         169/52007         939/52015        1743/51748        2073/51918       11555/199072       1620/17522   1
```

</details>

Let's break it down:

- The computer deviated from the basic strategy play on 11.8M out of 197M decisions. That works out to almost exactly
  **6%** of decisions.
- On average, the deviant plays earned a theoretical average of **0.449%** of expected value. Over 144M hands, the
  actual loss was only **0.049%** - around 5 cents for every $100 bet.
- Compared to a basic-strategy house edge of around 0.5%, the simulation and calculated EV gain agreed with each other.
  Brute-forcing predicted a gain of around 0.449%, and `0.5% - 0.449% ≈ 0.049%`.
- In this particular game, the computer **improved on the long-run performance of a basic strategy player by a factor of
  ten!**
- Taking the "Insurance" bet when the deck composition called for it accounted for about 0.08% of the EV gained, around
  1/6 of the total EV gain from all deviations.
- Looking at per-hand deviations, some make sense. Many hands have zero deviations because there is no draw pile extreme
  enough to tilt the odds away from the basic decision, such as hitting a hard 20 or 21.
- Borderline plays are noticeable on the per-hand chart. These agree with common Advantage Play deviations based on a
  simple running count of high and low cards left in the deck.
    - A hard 12 vs a 4-up deviates over 45% of the time.
    - A hard 16 vs a 10-up deviates around 40% of the time.
    - Perhaps the most infamous "dubious" play, splitting tens, is the correct choice against a 6-up around 12% of the
      time.

## Takeaways

- With slightly more favorable table rules, a computer could reasonably beat the house without ever adjusting its bet
  sizes. This specific simulation comes so close to a fair game that simply increasing deck penetration, allowing a few
  more doubles, or altering bets with a very small spread would be enough to overcome the house edge.
- **Do NOT try this for real money.** Card counting with your head is not illegal. Card counting with a program like
  this is very illegal.

## Performance

I also wanted to get a benchmark of pure hand simulation that did not involve any brute-forcing. Rust did its job
beautifully - the simulator is blazing fast when using a simple lookup table for its decision-making. These are the
results of using 20 threads on a Ryzen 5900X for about a minute:

```
Started 2573248431 hands (499288429 shoes, 2804642332 units placed) and had total of -14783846 returned.
Edge = -0.5271205469346814% (units returned/placed), 38982908 hands/sec total (39494662 hands/7663049 shoes in last
second), 0/3508287419 deviant actions 0% average +EV/hand. 0/0/197576910 insurances won/taken/offered (+0 EV).
```

Around *2.5 billion hands per minute*! I did not investigate any other Blackjack simulator programs out there, but I
would be surprised if anything else not written in Rust or C(++) even comes close.

Of course, that's not the performance we get when we start brute-forcing perfect strategies. As shown by the above
result that ran for several hours, it's much closer to 130,000 hands per minute. This makes sense - there is a massive
tree of outcomes to analyze, compounded especially by the ability to split hands. I memoized specific scenarios since
there is overlap in outcomes within a single running hand, but any further speedup would mean the computer's decision
being less "perfect".

## Try it Yourself

I did not write this program to have any kind of user interface. You will need to be familiar with compiling code to run
it and editing code to make changes.

Requires Rust and Cargo.

```bash
git clone https://github.com/joshuaprince/blackjack_composition.git
cd blackjack_composition
cargo build --release
./target/release/blackjack_composition
```

If you are programmatically inclined, here are some things to try tweaking that really ought to have a user interface:

- Simulate fast Basic Strategy instead of a comparison: In `main.rs::play_hands_compare_and_report`: Change the
  `PlayerDecisionMethod::BasicPerfectComparison` to `PlayerDecisionMethod::BasicStrategy`.
- Change the number of threads to use: `main.rs::THREADS` constant.
- Change the game rules: `rules.rs`. The rules struct is selected as the `main.rs::RULES` constant. WARNING: Not all
  combinations of rules will work properly.

## Other Thoughts

**This was created by Joshua Prince in 2023. It is licensed under the AGPL.** Any changes to the code must also be
open-source and licensed under the AGPL. I make no guarantees about the accuracy or functionality of any data or 
code in this repository.

Special thanks to [The Wizard of Odds](https://wizardofodds.com/) for
developing [calculators](https://wizardofodds.com/games/blackjack/hand-calculator/)
and [tools](https://wizardofodds.com/games/blackjack/strategy/calculator/) that I used frequently as a "ground truth"
while developing and debugging this simulator.
