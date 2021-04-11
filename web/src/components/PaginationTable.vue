<template>
  <b-container>
    <vuetable
      ref="vuetable"
      class="table-scroll"
      :api-mode="false"
      :css="css"
      :fields="fields"
      :per-page="perPage"
      :data-manager="dataManager"
      pagination-path="pagination"
      @vuetable:pagination-data="onPaginationData"
    />
    <b-row class="pagination">
    <b-col lg="8">
      <vuetable-pagination
        ref="pagination"
        :css="css.pagination"
        @vuetable-pagination:change-page="onChangePage"
      />
    </b-col>
    <b-col lg="4">
      <b-form-group class="perPageContainer" label-cols="8" label-cols-lg="8" label="Per Page:" laebl-for="perPageSelect">
        <b-form-select id="perPageSelect" size="sm" v-model="perPage" :options="options"></b-form-select>
      </b-form-group>
    </b-col>
    </b-row>
  </b-container>
</template>

<script>
import Vuetable from "vuetable-2/src/components/Vuetable";
import VuetablePagination from "vuetable-2/src/components/VuetablePagination";
import PaginationTableStyle from "@/assets/css/PaginationTableStyle.js";
import _ from "lodash";
import Papa from "papaparse";

export default {
  name: "PaginationTable",
  components: {
    Vuetable,
    VuetablePagination,
    PaginationTableStyle,
  },
  data() {
    return {
      css: PaginationTableStyle,
      perPage: 10,
      options: [10, 25, 50, 100],
      fields: []
    };
  },
  props: {
    projectId: String,
    dataset_type: String,
  },

  methods: {
    vuetable() {
      return this.$refs.vuetable
    },
    onPaginationData(paginationData) {
      this.$refs.pagination.setPaginationData(paginationData);
    },
    onChangePage(page) {
      this.$refs.vuetable.changePage(page);
    },
    async dataManager(sortOrder, pagination) {
      // Query endpoint for the page that is needed
      // Wait for return
      let page_data = await this.$http.get(
        `api/projects/${this.projectId}/pagination/${this.dataset_type}?page=${pagination.current_page}&per_page=${this.perPage}`
      );

      console.log(page_data);
      // Set fields
      let total = page_data.data.total;
      let local = page_data.data.data;
      this.fields = page_data.data.fields.split(",");

      let with_header = page_data.data.fields.concat("\n").concat(local);

      let parsed = Papa.parse(with_header, { header: true });

      // do pagination calculations
      pagination = this.$refs.vuetable.makePagination(
        total,
        this.perPage
      );

      // return data from endpoint
      return {
        pagination: pagination,
        data: parsed.data,
      };
    },
  },
};
</script>
<style>

.pagination {
  background: #f9fafb;
}
.table-scroll table {
  display: block;
  overflow-x: auto;
  overflow-y: scroll;
}

</style>
