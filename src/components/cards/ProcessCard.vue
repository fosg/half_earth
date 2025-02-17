<template>
<Card color="#ffffff" :isProcess="true" :class="{'is-new': isNew}">
  <template v-slot:header>
    <div>{{display.enumDisplay(output)}}</div>
    <img v-if="isNew" class="new-card-icon" src="/assets/new.svg" />
    <div v-tip="outputTip">{{produced.amount}}<img :src="icons[output]"> {{produced.emissions}}<img :src="icons.emissions"></div>
  </template>
  <template v-slot:figure>
    <img class="card-image" :src="`/assets/content/images/${image.fname}`" />
    <div class="process-limit-alert"
      v-if="maxShare < 20"
      v-tip="{icon: 'alert', text: t('Because of resource availability this process can only make up to {maxPercent}% of production. {suggestion}', {maxPercent: maxShare * 5, suggestion: process.mix_share > maxShare || changedMixShare > maxShare ? t('You should reallocate its points to other processes.') : ''})}"><img :src="icons.alert" /></div>

    <div class="opposers" v-if="opposersDetailed.length > 0">
      <img v-for="npc in opposersDetailed" v-tip="{text: t(`{name} is opposed to this. If you ban it, your relationship will improve by +<img src='{icon}' />.`, {name: t(npc.name), icon: icons.relationship}), icon: npc.name}" :src="icons[npc.name]">
    </div>
    <div class="supporters" v-if="supportersDetailed.length > 0">
      <img v-for="npc in supportersDetailed" v-tip="{text: t(`{name} supports this. If you implement it, your relationship will improve by +<img src='{icon}' />.`, {name: t(npc.name), icon: icons.relationship}), icon: npc.name}" :src="icons[npc.name]">
    </div>
  </template>
  <template v-slot:name>
    {{t(name)}}
  </template>
  <template v-slot:body>
    <div class="card-actions" v-if="!!this.$slots.actions">
      <slot name="actions"></slot>
    </div>

    <div class="process-mix">
      <div class="process-excess-alert"
        v-if="process.mix_share > maxShare || changedMixShare > maxShare"
        v-tip="{icon: 'alert', text: t(`This process can't produce this much because of feedstock or other limits. You should reallocate its points to other processes.`)}"><img :src="icons.alert" /></div>
      <div class="process-mix-percents" :class="{depleted: feedstockEstimate == 0}" v-tip="changeTip">
        <div class="process-mix-percent" :class="{before: hasChange}">{{process.mix_share*5}}%</div>
        <template v-if="hasChange">
          <img :src="icons.arrow_right" />
          <div class="process-mix-percent after" :class="{
            shrink: process.mix_share > changedMixShare,
            grow: process.mix_share < changedMixShare,
          }">{{changedMixShare*5}}%</div>
        </template>
      </div>
    </div>

    <div class="process-intensity space-even">
      <IntensityIcon
        v-tip="intensityTip('energy')"
        resource="energy" :intensity="intensities.energy" />
      <IntensityIcon
        v-tip="intensityTip('water')"
        resource="water" :intensity="intensities.water" />
      <IntensityIcon
        v-tip="intensityTip('biodiversity')"
        resource="extinction_rate" :intensity="intensities.biodiversity" />
      <IntensityIcon
        v-tip="intensityTip('land')"
        resource="land" :intensity="intensities.land" />
      <IntensityIcon
        v-tip="intensityTip('emissions')"
        resource="emissions" :intensity="intensities.emissions" />
    </div>
  </template>
  <template v-slot:top-back>
    <p class="card-desc">{{t(description)}}</p>
  </template>
  <template v-slot:bot-back>
    <div class="process-details">
      <div>
        <img v-if="feedstockEstimate && feedstockEstimate == 0" :src="icons.halted" class="alert-icon" />
        <img v-else-if="feedstockEstimate && feedstockEstimate < 20" :src="icons.alert" class="alert-icon" />
        <img v-if="feedstockName != 'other'"
          v-tip="{text: t(`This process uses {feedstockName}. {feedstockEstimateDesc}`, {feedstockName: t(feedstockName), feedstockEstimateDesc}), icon: feedstockIcon}"
          class="process-feedstock" :src="icons[feedstockIcon]">
        <div class="feedstock-remaining" v-if="feedstockName != 'other' && feedstockName != 'soil'">
          <div :class="`feedstock-remaining-fill feedstock-remaining-fill--${feedstockLevel}`"></div>
        </div>
      </div>
      <div>
        <img class="process--feature" v-for="feature in featureIcons" :src="icons[feature.icon]" v-tip="{icon: feature.icon, text: t(feature.text)}"/>
      </div>
    </div>
    <div class="card-spacer"></div>
    <div class="card-image-attribution">
      {{t('Image:')}} {{image.attribution}}
    </div>
  </template>

  <template v-slot:process-mix>
    <div class="process-mix-cells" v-tip="{icon: 'mix_token', text: maxShare < 20 ? t(`Because of resource availability this process can only make up to {maxPercent}% of production.`, {maxPercent: maxShare * 5}) : t(`There is currently no limit on this process' mix share.`)}">
        <div class="process-mix-cell" v-for="i in 20" :class="{
          active: i <= process.mix_share,
          depleted: feedstockEstimate == 0,
          shrink: i <= process.mix_share && i > changedMixShare,
          grow: i > process.mix_share && i <= changedMixShare,
          excess: (i <= process.mix_share || i <= changedMixShare) && i > maxShare,
          disabled: i > maxShare,
        }"/>
      </div>
  </template>
</Card>
</template>

<script>
import t from '/src/i18n';
import Card from './Card.vue';
import game from '/src/game';
import state from '/src/state';
import format from '/src/display/format';
import factors from '/src/display/factors';
import display from '/src/display/display';
import intensity from '/src/display/intensity';
import IntensityIcon from './IntensityIcon.vue';
import PROCESSES from '/assets/content/processes.json';
import NPCS from '/assets/content/npcs.json';
import icons from '/src/components/icons';

const FEATURE_DESCS = {
  'IsSolar': 'This process relies on the sun.',
  'IsIntermittent': 'This process is intermittent.',
  'CanMeltdown': 'This process can meltdown.',
  'MakesNuclearWaste': 'This process produces nuclear waste.',
  'IsLaborIntensive': 'This process is especially labor-intensive.',
  'IsCombustion': 'This process involves combustion.',
  'IsFossil': 'This process uses fossil fuels.',
  'IsCCS': 'This process captures and stores carbon.',
  'UsesOil': 'This process uses oil.',
  'UsesLivestock': 'This process uses livestock.',
  'UsesPesticides': 'This process use pesticides.',
  'UsesSynFertilizer': 'This process uses synthetic fertilizers.',
};

export default {
  props: ['process'],
  components: {
    Card,
    IntensityIcon,
  },
  mounted() {
    if (this.hasChange) {
      let i = 0;
      let els = [...this.$el.querySelectorAll('.process-mix-cell')];
      if (this.change < 0) {
        els.reverse();
      }
      els.forEach((el) => {
        if (el.classList.contains('shrink') || el.classList.contains('grow')) {
          el.style.animationDelay = `${i*0.25}s`;
          i++;
        }
      });
    }
  },
  data() {
    return {
      state,
      ...this.process,
      ...PROCESSES[this.process.id],
      output: display.enumKey(this.process.output),
    };
  },
  computed: {
    isNew() {
      return !state.viewed.includes(this.ref_id);
    },
    outputTip() {
      return {
        icon: this.output,
        text: t(`This process currently produces {amount}<img src='{outputIcon}'> and {emissions}<img src='{emissionsIcon}'> per year.`, {emissionsIcon: icons.emissions, outputIcon: icons[this.output], emissions: this.produced.emissions, amount: this.produced.amount})
      }
    },
    changeTip() {
      return {
        icon: 'mix_token',
        text: t(`This process currently makes up {mixPercent}% of {output} production.`, {output: this.output, mixPercent: this.process.mix_share*5})
      };
    },
    featureIcons() {
      return this.features.map((feat) => {
        return {
          icon: feat,
          text: t(FEATURE_DESCS[feat])
        };
      });
    },
    produced() {
      let baseAmount = state.gameState.produced_by_process[this.id];
      let amount = format.output(baseAmount, this.output);
      amount = amount > 0 ? Math.max(amount, 1) : amount;

      let emissions = format.gtco2eq(this.byproducts, baseAmount);
      emissions = emissions > 0 ? Math.max(emissions, 1) : emissions;
      return {
        emissions,
        amount
      };
    },
    maxShare() {
      return game.processMaxShare(this.process);
    },
    feedstockIcon() {
      return display.enumKey(this.feedstock[0]);
    },
    feedstockName() {
      return display.enumDisplay(this.feedstock[0]);
    },
    feedstockEstimate() {
      let feedstock = display.enumKey(this.feedstock[0]);
      if (feedstock == 'other' || feedstock == 'soil') {
        return null;
      }
      let estimate = state.gameState.feedstocks[feedstock]/state.gameState.consumed_feedstocks[feedstock];
      return Math.round(estimate);
    },
    feedstockEstimateDesc() {
      if (this.feedstockEstimate == null) {
        return '';
      } else if (this.feedstockEstimate == 0) {
        return t('This feedstock is depleted, so this process is stopped. You should reallocate its points to other processes.');
      } else if (isFinite(this.feedstockEstimate)) {
        return t(`At current usage rates the estimated supply is expected to last {years} years.`, {years: this.feedstockEstimate});
      } else {
        return t(`At current usage rates the estimated supply is expected to last indefinitely.`);
      }
    },
    feedstockLevel() {
      let feedstock = display.enumKey(this.feedstock[0]);
      if (feedstock == 'other' || feedstock == 'soil') {
        return 'high';
      } else if (this.feedstockEstimate < 20) {
        return 'low';
      } else if (this.feedstockEstimate < 50) {
        return 'mid';
      } else if (this.feedstockEstimate < 80) {
        return 'high';
      } else {
        return 'very-high';
      }
    },
    change() {
      return state.processMixChanges[this.process.output][this.process.id] || 0;
    },
    hasChange() {
      return this.change !== 0;
    },
    changedMixShare() {
      let change = state.processMixChanges[this.process.output][this.process.id] || 0;
      return this.process.mix_share + change;
    },
    intensities() {
      let type =
        (this.output == 'electricity' || this.output == 'fuel')
        ? 'energy' : 'calories';
      let values = {
        emissions: format.co2eq(this.byproducts),
        biodiversity: this.extinction_rate,
        energy: this.resources.electricity + this.resources.fuel,
        land: this.resources.land,
        water: this.resources.water,
      };
      let intensities = Object.keys(values).reduce((acc, k) => {
        acc[k] = intensity.intensity(values[k], k, type);
        return acc;
      }, {});
      return intensities;
    },
    supportersDetailed() {
      return this.supporters
        .filter((id) => !state.gameState.npcs[id].locked)
        .map((id) => NPCS[id]);
    },
    opposersDetailed() {
      return this.opposers
        .filter((id) => !state.gameState.npcs[id].locked)
        .map((id) => NPCS[id]);
    },
  },
  methods: {
    intensityTip(type) {
      switch (type) {
        case 'land': {
          let amount = format.landUsePercent(state.gameState.resources_demand.land);
          return factors.tips.land(
            t(`Land: They're not making anymore of it. You're using {percent}% of land.`, {percent: amount.toFixed(0)}),
            this.process);
        }
        case 'emissions': {
          let amount = state.gameState.world.emissions;
          return factors.tips.emissions(
            t(`Emissions: A shroud around the earth. You're emitting {amount} gigatonnes per year.`, {amount: amount.toFixed(1)}),
            this.process);
        }
        case 'water': {
          let amount = format.waterUsePercent(state.gameState.resources_demand.water);
          return factors.tips.water(
            t(`Water: The giver of life. You're using {percent}% of water resources.`, {percent: amount.toFixed(0)}),
            this.process);
        }
        case 'energy': {
          let amount = format.twh(state.gameState.output_demand.fuel + state.gameState.output_demand.electricity);
          return factors.tips.energy(
            t(`Energy: The fundamental mover. You're using {amount}TWh of energy.`, {amount}),
            this.process);
        }
        case 'biodiversity': {
          let amount = state.gameState.world.extinction_rate;
          return factors.tips.biodiversity(
            t(`Biodiversity: The co-inhabitants of the planet. The current biodiversity threat index is {amount}.`, {amount}),
            this.process);
        }
      }
    }
  }
}
</script>

<style>
.process-intensity {
  margin: 0.5em 0;
}

.process-intensity img{
  width: 24px;
  margin-bottom: 2px;
}

.process-trend,
.process-feedstock {
  width: 24px;
  background: #222;
  border-radius: 10em;
  padding: 0.35em 0.2em;
  border: 1px solid #888;
}
.process-feedstock {
  padding: 0.2em 0.2em;
}

.process-mix {
  display: flex;
  justify-content: center;
  /* margin-top:10px; */
}
.process-mix img {
  width: 18px;
  vertical-align: top;
}
.process-mix small{
  font-size:0.5em;
  margin: 0 0.5em;
}

.process-mix-percent {
  background: #111;
  color: #fff;
  padding: 0.2em 0.15em 0.1em;
  border-radius: 0.2em;
  font-size: 1.5rem;
  font-family: 'W95FA';
}

.process-mix-percent.before{
 color: #aaa;
}

.process-mix-percent.after.shrink{
color: #FF9A52;
}
.process-mix-percent.after.grow{
color: #63FF96;
}

.process-mix-percents {
  text-align: center;
  display: flex;
  align-items: center;
}
.process-mix-percents.depleted {
  color: #aaa;
}
.process-mix-cells{
  height: 100%;
  display: flex;
  flex-direction: column-reverse;
}
.process-mix-cell {
  height: calc(100%/20);
  width: 100%;
  margin: 1px;
  box-shadow: inset -1px -1px 0px rgb(0 0 0 / 50%);
  border-left: 1px solid rgba(255,255,255,0.5);
  border-top: 1px solid rgba(255,255,255,0.5);
  background-color: rgba(0,0,0,0.1);
}

.process-mix-cell:last-of-type{
}

.process-mix-cell:first-of-type{
}

.process-mix-cell.active {
  background: #1B97F3;
  box-shadow: 0 0 8px #1B97F3;
}
.process-mix-cell.active.depleted {
  background: #6190B3;
  box-shadow: 0 0 0px #6190B3;
}
.process-mix-cell.active.shrink {
  background: #F28435;
  box-shadow: 0 0 2px #F28435;
  animation-duration: 0.5s;
  animation-name: shrink-glow;
  animation-iteration-count: infinite;
  animation-direction: alternate;
}

.process-mix-cell.grow {
  background: #43CC70;
  box-shadow: 0 0 8px #43CC70;
  animation-duration: 0.5s;
  animation-name: grow-glow;
  animation-iteration-count: infinite;
  animation-direction: alternate;
}

@keyframes shrink-glow {
  from {
    box-shadow: 0 0 2px #f28435;
    opacity: 0.7;
  }
  to {
    box-shadow: 0 0 4px #f28435;
    opacity: 1.0;
  }
}

@keyframes grow-glow {
  from {
    box-shadow: 0 0 8px #43CC70;
    opacity: 0.7;
  }
  to {
    box-shadow: 0 0 12px #43CC70;
    opacity: 1.0;
  }
}

@keyframes excess-glow {
  from {
    box-shadow: 0 0 8px #DC322E !important;
    opacity: 0.7;
  }
  to {
    box-shadow: 0 0 12px #DC322E !important;
    opacity: 1.0;
  }
}

.process-mix-cell.excess {
  background: #DC322E !important;
  box-shadow: 0 0 8px #DC322E !important;
  opacity: 1 !important;
  animation-duration: 0.5s;
  animation-name: excess-glow;
  animation-iteration-count: infinite;
  animation-direction: alternate;
}
.process-mix-cell.disabled {
  background: #838383;
  opacity: 0.05;
}

.alert-icon {
	position: absolute;
	width: 16px;
	right: 0;
	bottom: 0;
	transform: translate(50%, 0);
}

.process-details {
  display: flex;
}

.feedstock-remaining {
  height: 5px;
  background: #aaa;
  width: 24px;
  border-radius: 1em;
  outline: 1px solid #555;
  margin-top: -2px;
  overflow: hidden;
}
.feedstock-remaining-fill {
  height: 100%;
}
.feedstock-remaining-fill--low {
  background: #EF3838;
  width: 20%;
}
.feedstock-remaining-fill--mid {
  background: #FBC011;
  width: 50%;
}
.feedstock-remaining-fill--high {
  background: #43CC70;
  width: 80%;
}
.feedstock-remaining-fill--very-high {
  background: #43CC70;
  width: 95%;
}

.process--feature {
  height: 24px;
  background: #222;
  border-radius: 1.2em;
  padding: 0.2em;
  margin-left: 2px;
  border: 1px solid #888;
}

.process-excess-alert {
  margin: 5px 2px 1px;
}

.process-mix-percents img {
  filter: invert(1);
}

.process-limit-alert {
  position: absolute;
  top: 5px;
  width: 28px;
  left: 5px;
  background: #ee7373;
  border-radius: 20em;
  padding: 4px 5px 2px 5px;
  border: 1px solid #222;
}
</style>
