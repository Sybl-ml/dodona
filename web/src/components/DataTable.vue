<template>
  <b-container fluid class="mt-3">
    <b-row>
      <b-col v-if="!data && !loading" class="text-center">
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
		<vuetable ref="vuetable"
			:api-mode="false"
			:fields="fields"
            :per-page="perPage"
			:data-manager="dataManager"
            pagination-path="pagination"
	            @vuetable:pagination-data="onPaginationData"
		/>
        <div style="padding-top:10px">
            <vuetable-pagination ref="pagination"
                @vuetable-pagination:change-page="onChangePage"
            />
        </div>
      </b-col>
    </b-row>
  </b-container>
</template>

<style scoped>
.input-table {
  height: calc(50px * 12);
  overflow-y: scroll;
}
</style>

<script>

import Vuetable from 'vuetable-2'

export default {
  name: "DataTable",
  props: {
    projectId: String,
    data: Object,
    loading: Boolean,
  },
  components: {
    'vuetable-pagination': Vuetable.VuetablePagination
  },
  data() {
    return {
      fields: FieldsDef,
      perPage: 5,
      data: this.data
    };
  },
  watch: {
    data(newVal, oldVal) {
      this.$refs.vuetable.refresh();
    }
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
        console.log("orderBy:", sortOrder[0].sortField, sortOrder[0].direction);
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
      console.log('pagination:', pagination)
      let from = pagination.from - 1;
      let to = from + this.perPage;

      return {
        pagination: pagination,
        data: _.slice(local, from, to)
      };
    },
  }
};
</script>
