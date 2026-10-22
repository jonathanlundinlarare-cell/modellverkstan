//! Exempel för samhällskunskap: privatekonomi och samhällsekonomi.

use super::Builder;
use crate::model::{Model, ModelKind, NodeKind};

pub fn savings() -> Model {
    let mut b = Builder::new(
        ModelKind::Sfd,
        "Sparande och ränta på ränta",
        "Pengarna på kontot är ett LAGER. Två FLÖDEN fyller på: dina egna insättningar och räntan. \
         Räntan räknas på allt som redan finns på kontot – även på tidigare ränta. Det kallas ränta på ränta, \
         och det är en förstärkande loop: ju mer pengar, desto mer ränta, desto mer pengar. \
         Det nedre lagret räknar bara hur mycket du själv har satt in, så att du kan jämföra.",
        &[
            "Kör modellen. Hur stor del av pengarna efter 40 år är ränta och hur stor del har du satt in själv?",
            "Sätt räntan till 0 %. Hur ser kurvorna ut nu? Varför?",
            "Jämför att spara i 20 år med att spara i 40 år (ändra under Tid och simulering). Blir pengarna dubbelt så många?",
            "Vad är bäst: 500 kr i månaden i 40 år, eller 1 000 kr i månaden i 20 år?",
        ],
    );
    let money = b.calc(NodeKind::Stock, "Pengar på kontot", [460.0, 250.0], "Startbelopp", "kr", "Allt som finns på kontot.");
    let monthly = b.constant("Månadssparande", [200.0, 110.0], 500.0, (0.0, 3000.0), "kr/månad", "Hur mycket du sätter in varje månad.");
    let rate = b.constant("Årsränta", [720.0, 90.0], 5.0, (0.0, 12.0), "%", "Räntan per år. En fond kan ge mer, men det är inte säkert.");
    let start = b.constant("Startbelopp", [720.0, 330.0], 0.0, (0.0, 100_000.0), "kr", "Pengar som redan finns när du börjar.");
    b.flow("Insättningar", [220.0, 250.0], None, Some(money), "Månadssparande * 12", "kr/år", "Det du själv sätter in per år.");
    b.flow("Ränta", [460.0, 80.0], None, Some(money), "Pengar på kontot * Årsränta / 100", "kr/år", "Räntan räknas på allt som finns på kontot.");
    let own = b.calc(NodeKind::Stock, "Egna insättningar totalt", [460.0, 470.0], "Startbelopp", "kr", "Summan av allt du själv har satt in.");
    b.flow("Insatt", [220.0, 470.0], None, Some(own), "Månadssparande * 12", "kr/år", "");
    let interest = b.calc(NodeKind::Aux, "Tjänad ränta", [720.0, 470.0], "Pengar på kontot - Egna insättningar totalt", "kr", "Det som räntan har gett.");
    let _ = (monthly, rate, start, interest);
    b.graph(&[money, own]);
    b.model.dt = 0.05;
    b.model.t_end = 40.0;
    b.model.time_unit = "år".into();
    b.model
}

pub fn household_budget() -> Model {
    let mut b = Builder::new(
        ModelKind::Sfd,
        "Hushållets budget",
        "Sparbufferten är ett LAGER. Inkomsterna fyller på och utgifterna tömmer. \
         Blir det pengar över växer bufferten, annars krymper den – och under noll betyder att man har skulder. \
         I modellen blir man arbetslös en viss månad och får då ersättning från a-kassan, som är lägre än lönen.",
        &[
            "Kör modellen. Hur mycket finns i bufferten när personen blir arbetslös? Hur länge räcker den sedan?",
            "Vilka utgifter skulle du dra ner på för att bufferten inte ska ta slut? Prova!",
            "Hyran höjs med 1 500 kr i månaden. Hur påverkar det?",
            "Hur stor buffert behövs för att klara ett år som arbetslös?",
        ],
    );
    let buffer = b.calc(NodeKind::Stock, "Sparbuffert", [470.0, 260.0], "Startbuffert", "kr", "Pengar på kontot. Under noll = skulder.");
    let salary = b.constant("Lön efter skatt", [90.0, 40.0], 25_000.0, (0.0, 50_000.0), "kr/månad", "Det som kommer in på kontot varje månad.");
    let jobless = b.constant("Arbetslös från månad", [330.0, 30.0], 12.0, (0.0, 24.0), "", "Den månad då personen blir arbetslös.");
    let benefit = b.constant("Ersättning", [90.0, 170.0], 65.0, (0.0, 100.0), "% av lönen", "A-kassan ger en del av lönen.");
    let income = b.calc(
        NodeKind::Aux,
        "Inkomst",
        [250.0, 120.0],
        "om(tid < Arbetslös från månad; Lön efter skatt; Lön efter skatt * Ersättning / 100)",
        "kr/månad",
        "Lön eller ersättning från a-kassan.",
    );
    b.flow("Inkomster", [250.0, 260.0], None, Some(buffer), "Inkomst", "kr/månad", "");
    let rent = b.constant("Hyra", [640.0, 70.0], 8_000.0, (0.0, 20_000.0), "kr/månad", "");
    let food = b.constant("Mat och övrigt", [840.0, 130.0], 10_000.0, (0.0, 20_000.0), "kr/månad", "Mat, el, telefon, kläder, resor …");
    let fun = b.constant("Nöjen", [840.0, 380.0], 4_000.0, (0.0, 10_000.0), "kr/månad", "Restaurang, streaming, hobbyer …");
    b.flow("Utgifter", [690.0, 260.0], Some(buffer), None, "Hyra + Mat och övrigt + Nöjen", "kr/månad", "");
    let left = b.calc(NodeKind::Aux, "Över per månad", [470.0, 100.0], "Inkomster - Utgifter", "kr/månad", "Plus = sparande, minus = bufferten krymper.");
    let start = b.constant("Startbuffert", [470.0, 430.0], 20_000.0, (0.0, 100_000.0), "kr", "Sparade pengar från början.");
    let _ = (salary, jobless, benefit, rent, food, fun, left, start);
    b.graph(&[buffer, income]);
    b.model.dt = 0.05;
    b.model.t_end = 24.0;
    b.model.time_unit = "månader".into();
    b.model
}

pub fn loan() -> Model {
    let mut b = Builder::new(
        ModelKind::Sfd,
        "Lån och skuld",
        "Skulden är ett LAGER. Räntan gör skulden större varje månad (ett inflöde) och avbetalningarna gör den mindre (ett utflöde). \
         Om avbetalningen är mindre än räntan växer skulden fast man betalar! \
         Ett SMS-lån kan ha en årsränta på över 30 %, ett billån kanske 6–10 %.",
        &[
            "Kör modellen. Hur många månader tar det att betala tillbaka lånet? Hur mycket ränta betalar man totalt?",
            "Gör om lånet till ett SMS-lån: sätt årsräntan till 39 %. Hur mycket dyrare blir det?",
            "Med 39 % ränta: sänk månadsbetalningen till 300 kr. Vad händer med skulden? Varför?",
            "Vad lönar sig mest: att betala lite mer varje månad eller att hitta ett lån med lägre ränta?",
        ],
    );
    let debt = b.calc(NodeKind::Stock, "Skuld", [470.0, 260.0], "Lånebelopp", "kr", "Det som återstår att betala.");
    let amount = b.constant("Lånebelopp", [470.0, 430.0], 10_000.0, (1_000.0, 100_000.0), "kr", "Hur mycket man lånar.");
    let rate = b.constant("Årsränta", [240.0, 100.0], 8.0, (0.0, 40.0), "%", "Räntan per år.");
    let payment = b.constant("Månadsbetalning", [700.0, 100.0], 500.0, (100.0, 5_000.0), "kr/månad", "Hur mycket man betalar tillbaka varje månad.");
    let interest = b.flow(
        "Ränta",
        [240.0, 260.0],
        None,
        Some(debt),
        "Skuld * Årsränta / 100 / 12",
        "kr/månad",
        "Räntan för en månad är en tolftedel av årsräntan.",
    );
    b.flow(
        "Avbetalning",
        [700.0, 260.0],
        Some(debt),
        None,
        "min(Månadsbetalning; max(0; Skuld) * 20)",
        "kr/månad",
        "Man betalar månadsbeloppet tills skulden är borta.",
    );
    let paid = b.calc(NodeKind::Stock, "Betald ränta", [470.0, 580.0], "0", "kr", "All ränta som har betalats hittills.");
    b.flow("Räntekostnad", [240.0, 580.0], None, Some(paid), "Ränta", "kr/månad", "");
    let total = b.calc(NodeKind::Aux, "Lånet kostar totalt", [700.0, 580.0], "Lånebelopp + Betald ränta", "kr", "Lånebeloppet plus all ränta.");
    let _ = (amount, rate, payment, interest, total);
    b.graph(&[debt, paid]);
    b.model.dt = 0.05;
    b.model.t_end = 60.0;
    b.model.time_unit = "månader".into();
    b.model
}

pub fn economic_cycle() -> Model {
    let mut b = Builder::new(
        ModelKind::Cld,
        "Det ekonomiska kretsloppet",
        "När hushållen handlar säljer företagen mer. Då behövs fler anställda, och fler jobb ger hushållen mer pengar att handla för – \
         en FÖRSTÄRKANDE loop. Men en del av inkomsterna sparas eller går till skatt i stället för att handlas för, \
         vilket bromsar (BALANSERANDE loopar). Skatterna används till offentliga utgifter som skola och vård, som också ger jobb.",
        &[
            "Följ loop R1 runt. Vad händer i hela kretsloppet om hushållen plötsligt handlar mindre (t.ex. i en kris)?",
            "Varför kallas sparande och skatter för \"läckage\" ur kretsloppet? Hitta de balanserande looparna.",
            "Riksbanken höjer räntan. Följ pilarna: hur påverkas jobben?",
            "Lägg till \"Export\" och \"Import\". Var ska de kopplas in, och med + eller −?",
        ],
    );
    let consumption = b.var("Hushållens konsumtion", [450.0, 40.0]);
    let sales = b.var("Företagens försäljning", [790.0, 160.0]);
    let jobs = b.var("Antal jobb", [730.0, 450.0]);
    let income = b.var("Hushållens inkomster", [380.0, 540.0]);
    let saving = b.var("Sparande", [110.0, 170.0]);
    let taxes = b.var("Skatter", [90.0, 410.0]);
    let public = b.var("Offentliga utgifter", [560.0, 310.0]);
    let interest = b.var("Räntan", [210.0, 40.0]);
    b.link(consumption, sales, true, 0.2);
    b.link(sales, jobs, true, 0.2);
    b.link(jobs, income, true, 0.2);
    b.link(income, consumption, true, -0.15);
    b.link(income, saving, true, -0.2);
    b.link(saving, consumption, false, -0.2);
    b.link(income, taxes, true, 0.2);
    b.link(taxes, consumption, false, 0.0);
    b.link(taxes, public, true, 0.15);
    b.link(public, jobs, true, 0.15);
    b.link(interest, saving, true, 0.0);
    b.model
}
