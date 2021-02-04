<template>
  <b-tabs pills card vertical v-model="tabIndex">
      <b-tab
        v-for="data in tabs"
        v-bind:key="data.key"
        :title="data.title"
        :disabled="data.disabled"
        :active="data.key=='1'"
      >
        <!-- CHILD COMPONENT -->
        <slot :name="data.key"/>
        
        <b-button
          v-if="data.key != 1"
          size="sm"
          class="mb-3 float-left"
          variant="primary"
          @click="tabIndex--"
          ><strong>Previous</strong></b-button
        >
        <b-button
          v-if="data.key != tabs.length"
          size="sm"
          class="mb-3 float-right"
          variant="primary"
          @click="tabIndex++"
          ><strong>Next</strong></b-button
        >
      </b-tab>
    </b-tabs>
</template>

<script>
export default {
  name: "NavigatableTab",
  props: {
    tabs: Array,
  },
  data() {
    return {
      tabIndex: 1,
    };
  },
};
</script>
