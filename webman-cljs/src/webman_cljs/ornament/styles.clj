(ns webman-cljs.ornament.styles
  (:require [lambdaisland.ornament :as o]
            [garden.selectors :refer [nth-child &]]
            [garden.stylesheet :refer [at-keyframes]]))

(def global-styles
  [[:html {:font-size "14pt"}]
   (at-keyframes "lds-grid"
                 ["0%, 100%" {:opacity 1}]
                 ["50%" {:opacity 0.5}])])

(defn prefix-key [prefix k] (keyword (str prefix (name k))))
(defn prefix-map [prefix m] (map (fn [[k v]] [(prefix-key prefix k) v]) m))
(defn ld
  "generate light dark classes"
  [prefix color]
  [(prefix-key (str (name prefix) "-l") color)
   (prefix-key (str "dark:" (name prefix) "-d") color)])

(defn round [] (str (with-precision 5 (/ 1 3M))))

(defn set-token
  []
  (let [light {:main "FFFFFF",
               :shadow "f8f8f8",
               :base "FFB2B2",
               :accent "257BFF",
               :text "010101"}
        dark {:main "2D2727",
              :shadow "413543",
              :base "8B8080",
              :accent "ffff36",
              :text "D2D8D8"}
        colors (into {}
                     (concat (prefix-map "l" light)
                             (prefix-map "d" dark)))]
    (o/set-tokens!
     {:colors colors,
      :components
      [{:id :c-bg, :garden (into [:&] (ld :bg :main))}
       {:id :c-shadow, :garden (into [:&] (ld :bg :shadow))}
       {:id :c-border, :garden (into [:&] (ld :border :text-60))}
       {:id :c-border-inverse,
        :garden [:& :border-dtext :dark:border-ltext]}
       {:id :c-accent-text, :garden (into [:&] (ld :text :accent))}
       {:id :c-accent,
        :garden (into [:&] (concat (ld :bg :accent) (ld :text :main)))}
       {:id :grid-loading,
        :rules
        "grid-loading = <'grid-loading-'> size
         <size> = #'[0-9]+'",
        ;; the style of grid in css-spinner
        :garden (fn [{[size] :component-data}]
                  (let [size (bigdec size)
                        px (fn [n] (str (int (bigdec n)) "px"))
                        round (fn [n] (with-precision 5 n))
                        d (fn [div] (round (/ size div)))
                        ball-size (d 5)
                        gap-size (d 10)
                        speed 1.2]
                    [:&
                     {:box-sizing "border-box",
                      :display "inline-block",
                      :position "relative",
                      :width (px size),
                      :height (px size)}
                     [:div
                      {:box-sizing "border-box",
                       :position "absolute",
                       :width (px ball-size),
                       :height (px ball-size),
                       :border-radius "50%",
                       :animation (format "lds-grid %.1fs linear infinite"
                                          speed)}
                      (for [n (range 9)]
                        [(& (nth-child (str (+ n 1))))
                         (let [row (Math/floor (/ n 3))
                               col (mod n 3)]
                           {:top (px (round
                                      (+ gap-size
                                         (* row (+ ball-size gap-size))))),
                            :left (px (round (+ gap-size
                                                (* col
                                                   (+ ball-size
                                                      gap-size))))),
                            :animation-delay (str (* speed
                                                     (/ -1 3)
                                                     (mod (+ col row) 3))
                                                  "s")})])]]))}
       {:id :c-base, :garden (into [:&] (ld :bg :base))}
       {:id :c-text, :garden (into [:&] (ld :text :text))}
       {:id :c-secondary-text,
        :garden [:& :text-ltext-60 :dark:text-dtext-60]}]})))


