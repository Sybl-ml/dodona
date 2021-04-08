<template>
  <b-container>
    <vuetable
      ref="vuetable"
      class="table-scroll"
      :api-url="`localhost:3001/api/projects/${this.projectId}/pagination/${this.type}`"
      :css="css"
      :fields="['Time', 'Year']"
      data-path="data"
      :query-params="{sort: 'sort',page: 'page',perPage: 'per_page'}"
      :per-page="perPage"
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

export default {
  name: "ApiPaginationTable",
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
    };
  },
  props: {
    type: String,
    projectId: String
  },

  methods: {
    onPaginationData(paginationData) {
      this.$refs.pagination.setPaginationData(paginationData);
    },
    onChangePage(page) {
      this.$refs.vuetable.changePage(page);
    },
    dataManager(sortOrder, pagination) {
      if (this.data.length < 1) return;

      let local = this.data;

      // sortOrder can be empty, so we have to check for that as well
      if (sortOrder.length > 0) {
        local = _.orderBy(
          local,
          sortOrder[0].sortField,
          sortOrder[0].direction
        );
      }

      pagination = this.$refs.vuetable.makePagination(
        local.length,
        this.perPage
      );
      
      let from = pagination.from - 1;
      let to = from + this.perPage;

      return {
        pagination: pagination,
        data: _.slice(local, from, to),
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
