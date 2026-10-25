# Les mots de la surface de travail — Français.

panel-title-left = Panneau gauche
panel-title-right = Panneau droit
panel-column-name = Nom
panel-column-size = Taille
panel-column-date = Date
panel-column-type = Type
panel-type-file = Fichier
panel-type-directory = Dossier
panel-type-link = Lien
panel-type-other = Autre
panel-empty = Le dossier est vide
panel-status-ready = Prêt
panel-status-entries = { $count_number ->
        [one] 1 entrée
       *[other] { $count } entrées
    }
panel-status-selected = { $count_number ->
        [one] 1 sélectionné
       *[other] { $count } sélectionnés
    }
panel-status-filtered = { $count } sur { $total } entrées
panel-filter-label = Filtre
panel-filter-placeholder = motif de nom
panel-filter-matches = { $count_number ->
        [one] 1 correspondance
       *[other] { $count } correspondances
    }
panel-filter-grammar = * n'importe quels caractères · ? exactement un · ~ insensible à la casse
panel-tab = Onglet { $number }
panel-view-details = Détaillé
panel-view-compact = Compact
panel-focus = Panneau actif
panel-undo-notice = L'annulation supprime uniquement les copies ; l'état précédent n'est pas restauré.

# Les clés qui manquaient encore — la parité avec l'anglais est complète.
job-kind-copy = Copie
job-kind-create-directory = Création de dossier
job-kind-delete = Suppression
job-kind-move = Déplacement
job-kind-rename = Renommage
job-kind-sync = Synchronisation
job-state-cancelled = Annulé
job-state-done = Terminé
job-state-failed = Échoué
job-state-queued = En file d’attente
job-state-running = En cours
job-state-verifying = Vérification
job-state-waiting = En attente
menu-application = Application
menu-file = Fichier
menu-panel = Panneau
menu-view = Affichage
panel-hint-navigation = Tabulation pour le panneau, Entrée pour ouvrir, Espace pour marquer
panel-job-list-empty = Aucun travail
panel-job-list-title = Travaux
panel-job-progress = Progression : { $done } sur { $total }
panel-layout-side-by-side = Côte à côte
panel-layout-stacked = Superposés
panel-quick-view-binary = Fichier binaire — aucun aperçu
panel-quick-view-empty = Marquez un fichier pour voir son aperçu
panel-quick-view-title = Aperçu rapide
panel-tab-close = Fermer l’onglet
panel-tree-title = Arborescence

# The messages of the errors of the surface.

error-binding-empty = La liaison est vide.
error-binding-key-twice = La liaison nomme la touche { $key } deux fois.
error-binding-no-key = La liaison ne nomme aucune touche.
error-binding-empty-palette-entry = L'entrée de la palette d'actions ne porte aucun nom.
error-binding-unknown-action = L'action { $identifier } ne fait pas partie du catalogue.
error-binding-unknown-key = La liaison { $binding } nomme la touche inconnue { $key }.
error-transfer-same-panel = Le panneau { $side } ne peut pas être à la fois la source et la destination d'un transfert.
