<template>
  <b-container fluid >
    <b-row>
      <h4>{{this.datasetName}}
      </h4>
    </b-row>
    <b-row>
      <b-col v-if="!training_data && !loading" class="text-center">
        <b-row class="head-input-table">
        <b-table hover striped :items="this.dataHead.data" />
      </b-row>
      <b-button @click="$emit('get-data')" variant="primary" class="px-5"
        >Load Data</b-button
      >
      </b-col>
      <b-col v-else-if="loading" class="text-center">
        <b-icon
          icon="arrow-counterclockwise"
          animation="spin-reverse"
          font-scale="4"
        ></b-icon>
      </b-col>
      <b-col v-else class="input-table">
            <my-vue-table
              :fields="buildFields(this.training_data.meta.fields)"
              :display_data="this.training_data.data"
            ></my-vue-table>
      </b-col>
    </b-row>
  </b-container>
</template>

<style scoped>
.input-table {
  height: calc(50px * 12);
  overflow-y: scroll;
}
.head-input-table {
  height: calc(52px * 6);
}
</style>

<script>
import MyVueTable from "./MyVueTable.vue";
import VuetableFieldHandle from 'vuetable-2/src/components/VuetableFieldHandle.vue';

export default {
  name: "ProjectInput",
    components: {
    MyVueTable,
  },
  props: {
    projectId: String,
    datasetName: String,
    training_data: Object,
    dataHead: Object,
    loading: Boolean,
  },
  methods: {
    buildFields(fields) {
      let built_fields = [{
          name: VuetableFieldHandle
        }]
      
      fields.forEach(function (item, index) {
        built_fields.push({
            name: item,
            title: `<span class="orange glyphicon glyphicon-user"></span> ${item}`,
            sortField: item
          });
      });
      return built_fields;
    }
  },
  computed: {
    getDatasetDate() {
      return `${this.dataDate.toLocaleString("en-GB", {
        dateStyle: "short",
      })} - ${this.dataDate.toLocaleString("en-GB", {
        timeStyle: "short",
      })}`;
    },
  },
};
</script>
