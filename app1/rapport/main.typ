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

Les pins GPIO et UART du ESP32 utilisé pour les capteurs sont quasiment tous utilisés par les capteurs existants. Cela limite les possibilités d'ajouter de nouveaux capteurs utilisant ces interfaces. Pour ajouter de nouveaux capteurs, il faut donc utiliser le bus I2C.

Le bus I2C permet de connecter plusieurs capteurs utilisant la même interface et les mêmes fils, ce qui est idéal pour ajouter des capteurs à la station météorologique dans le futur.

Le seul problème, c'est  qu'il faut faire attention à ce que les capteurs connectés au bus I2C aient des adresses différentes pour éviter les conflits d'adresses. Dans le cas où deux capteurs ont la même adresse I2C, l'utilisation d'un multiplexeur I2C est envisageable si le changement d'adresse est un processus trop complexe ou même impossible.

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

Pour la problèmatique, il a été possible de brancher tous les capteurs au ESP32, mais l'ajout de plus de capteurs spécialisés pourrait remplir toutes les pins disponibles.

La solution idéale est d'utiliser uniquement l'interface I2C, étant donné la grande quantitée de clients pouvant utiliser le même bus I2C. L'utilisation de capteurs I2C est aussi plus simple à implémenter que d'autres interfaces, ce qui est un avantage pour la maintenance du code et l'ajout de nouveaux capteurs dans le futur. Par exemple, le capteur d'humidité et de température DHT11 utilise un protocole de communication propriétaire, ce qui rend son utilisation plus complexe et plus gourmande en ressources que les capteurs utilisant une interface standard telle que I2C. Nous avons eux des problèmes itermitents de décodage du signal du DHT11, où un capteur similaire utilisant I2C aurait permi une communication plus simple et plus fiable.

== Réseaux sans fil

Pour le réseau sans fil, plusieurs choix sont possibles dépendant des besoins de l'application.

=== BLE

Si la distance sans fils à couvrir est de 160m ou moins, l'antenne BLE du ANNA-B112 est déjà tout ce qu'il faut. Si on veut un peu plus de distance, on peut utiliser une antenne externe, ce qui est aussi supporté par le module.

La technologie BLE permet à une station de base de recevoir des données de plusieurs station météo différentes. Le nombre de stations météos pouvant être connectées à une station de base est limité par la capacité des driver BLE. Cependant cela peut être une grande quantitée si l'interval de communication est augmenté entre les stations météo et la station de base, laissant plus de temps pour les stations météo de transmettre leurs données à différents moments mais nécessite de l'orchestration et la déconnection/reconnection des stations météo.

=== LoRa

La technologie LoRa permet de couvrir des distances plus longues que BLE, mais nécessite une antenne externe et une configuration plus complexe. Si l'on veut que la station de base communique à plusieurs stations météo, il faut orchestrer la communication entre les stations et la station de base au niveau du code, ce qui n'est pas nécessairement difficile compte tenu que c'est la station de base qui gère la communication (envoie d'une seule requête à la fois et laisse le temps de recevoir la réponse).

L'utilisation d'une antenne LoRa externe est cependant nécessaire. La beautée du système est le fait qu'il n'y as pas de "connections" officielles entre les stations météo et la station de base, ce qui permet d'avoir un nombre très élevé de stations météo connectées à la station de base. L'utilisation du module STM32WL54CC serait une des solutions possibles pour implémenter ce système.

La distance parcourue peut être de même des kilomêtres si nécessaire et, compte tenu de sa basse fréquence, elle peut passer à travers des obstacles sans problèmes.

#pagebreak()
=== Zigbee

La technologie Zigbee est une autre option pour le réseau sans fil, offrant une portée plus longue que BLE et une consommation d'énergie plus faible que LoRa. Cependant, elle nécessite également une configuration plus complexe et l'utilisation d'une antenne externe. En contrepartie, elle permet une communication bidirectionnelle sécurisée entre les stations météo et la station de base.

== Choix de réseau sans fil

Le choix se résume donc au nombre de stations météo connectées à la station de base et à leurs distances respectives. Pour une utilisation domestique, le protocole Zigbee est idéal car il offre une portée suffisante et une consommation d'énergie réduite. Il permet aussi à capteurs de se connecter à un réseau Zigbee existant d'une autre marque de façon transparente. Pour une utilisation à plus grande échelle, le protocole LoRa est préférable car il offre une portée plus longue et la possibilité de connecter un plus grand nombre de stations météo à la station de base.
