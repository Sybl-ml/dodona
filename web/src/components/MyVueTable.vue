<template>
  <div>
    {{display_data}}
    <vuetable ref="vuetable"
      :api-mode="false"
      :data="display_data"
      :fields="fields"
      pagination-path=""
      :per-page="5"
      @vuetable:pagination-data="onPaginationData"
    ></vuetable>

    <div class="pagination ui basic segment grid">
      <vuetable-pagination-info ref="paginationInfo"
      ></vuetable-pagination-info>
      
      <vuetable-pagination ref="pagination"
        @vuetable-pagination:change-page="onChangePage"
      ></vuetable-pagination>
    </div>
  </div>
</template>

<script>
import Vuetable from "vuetable-2/src/components/Vuetable";
import VuetablePagination from "vuetable-2/src/components/VuetablePagination";
import VuetablePaginationInfo from 'vuetable-2/src/components/VuetablePaginationInfo';

export default {
  name: "MyVuetable",
  components: {
    Vuetable,
    VuetablePagination,
    VuetablePaginationInfo
  },
  props: {
    display_data: String,
    fields: Array,
  },

  methods: {
    onPaginationData(paginationData) {
      this.$refs.pagination.setPaginationData(paginationData);
      this.$refs.paginationInfo.setPaginationData(paginationData);
    },
    onChangePage(page) {
      this.$refs.vuetable.changePage(page);
    },
  }
};
</script>

<style>
  .pagination {
    margin-top: 1rem;
  }
</style>
