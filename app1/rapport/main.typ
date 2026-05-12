#import "template.typ": *
#show: template.with(
  titre: "Station météorologique",
  cours: "Physique des matériaux et capteurs",
  code: "GIF680",
  auteurs: (
    (nom: "Poulin-Bergevin, Charles", cip: "POUC1302"),
    (nom: "Stéphenne, Laurent", cip: "STEL2002"),
  ),
  date: "13 mai 2026",
  auteurs_footer: true,
)

= Possibilités d'utilisation de nouveaux capteurs

TODO voir si il rested des gpio/i2c disponibles (sike I2C peut avoir une chier d'addresses différentes sur le meme bus, si jamais 2 addresses pareil, utiliser un multiplexeur I2C)

= Changements à apporter pour une version commerciale

== Microcontrôleur

TODO voir si version moins chere (package ANA112B)

== Interface des capteurs

// TODO HEIN?

== Réseaux sans fil

TODO parler de zigbee (fuck that, LoraWan is better, juste à dire qu'on utilise un chip LoRaWAN de stm32, ça se trouve easy)
