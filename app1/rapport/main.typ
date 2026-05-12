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

// TODO voir si il rested des gpio/i2c disponibles (sike I2C peut avoir une chier d'addresses différentes sur le meme bus, si jamais 2 addresses pareil, utiliser un multiplexeur I2C)

= Changements à apporter pour une version commerciale

== Microcontrôleur

Le microcontrôleur est le coeur de nos 2 stations, autant la station météo que la station de base l'utilisent pour faire l'aggrégation des données, communiquer et coordonner leurs actions.

=== ESP32

Le microcontrôleur utilisé pour le prototype est le esp_wroom_32UE. Ce microcontrôleur est un module ESP32 équipé d'un port pour l'antenne pour le wifi et le bluetooth et, comme la plupart des SoC sur le marché, contient toutes les composantes nécessaires pour fonctionner sans composantes externes.

Le prototype actuel possède beaucoup de puissance de calcul et de mémoire, ce qui le rend malheureusement plus gourmand, en plus de ne pas inclure de fonctionnalités telle qu'une RTC pour le suivi du temps et permettre d'éteindre complètement le microcontrôleur pour une certaine durée de temps, diminuant ainsi drastiquement la consommation d'énergie.

=== ANNA-B112

Le SoC recommandé pour la version commerciale est le ANNA-B112. Le microcontrôleur utilisé à l'intérieur est le nrf5832. C'est un microcontrôleur de la famille nrf5 de Nordic Semiconductor, reconnus pour sa capacité à consommer extrêment peu d'énergie et sa robustesse. Voici les principaux arguments pour choisir cet SoC :

- Les 25 GPIO, le support pour UART, SPI, I2C, I2S, BLE, PDM et PWM et la résolution 12 bits de l'ADC ne limitent pas du tout sa capacitée de s'interfacer avec les capteurs.
- L'inclusion d'une RTC (Horloge à temps réel) permet d'éteindre le microcontrôleur pour une certaine durée de temps, réduisant ainsi la consommation d'énergie.
- Il nécessite un crystal externe pour la génération de l'horloge du microcontrôleur mais cela n'est pas un problème.
- La RTC du ANNA-B112 permet même de couper certaines GPIO lorsqu'elle éteint le microcontrôleur, éteignant ainsi les capteurs, réduisant encore plus la consommation d'énergie.

== Interface des capteurs

// TODO genre can vs spi vs 12c vs uart vs just plain pin reading (pls just all on i2c except weird ones, really simpler)

== Réseaux sans fil

Pour le réseau sans fil, plusieurs choix sont possibles dépendant des besoins de l'application.

=== BLE

Si la distance sans fils à couvrir est de 160m ou moins, l'antenne BLE du ANNA-B112 est déjà tout ce qu'il faut. Si on veut un peu plus de distance, on peut utiliser une antenne externe, ce qui est aussi supporté par le module.

La technologie BLE permet à une station de base de recevoir des données de plusieurs station météo différentes. Le nombre de stations météos pouvant être connectées à une station de base est limité par la capacité des driver BLE. Cependant cela peut être une grande quantitée si l'interval de communication est augmenté entre les stations météo et la station de base, laissant plus de temps pour les stations météo de transmettre leurs données à différents moments mais nécessite de l'orchestration et la déconnection/reconnection des stations météo.

=== LoRa

La technologie LoRa permet de couvrir des distances plus longues que BLE, mais nécessite une antenne externe et une configuration plus complexe. Si l'on veut que la station de base communique à plusieurs stations météo, il faut orchestrer la communication entre les stations et la station de base au niveau du code, ce qui n'est pas nécessairement difficile compte tenu que c'est la station de base qui gère la communication (envoie d'une seule requête à la fois et laisse le temps de recevoir la réponse).

L'utilisation d'une antenne LoRa externe est cependant nécessaire. La beautée du système est le fait qu'il n'y as pas de "connections" officielles entre les stations météo et la station de base, ce qui permet d'avoir un nombre très élevé de stations météo connectées à la station de base. L'utilisation du module STM32WL54CC serait une des solutions possibles pour implémenter ce système.

La distance parcourue peut être de même des kilomêtres si nécessaire et, compte tenu de sa basse fréquence, elle peut passer à travers des obstacles sans problèmes.

== Choix de réseau sans fil

Le choix se résume donc au nombre de stations météo connectées à la station de base et à leurs distances respectives.
