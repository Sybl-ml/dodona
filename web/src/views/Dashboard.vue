<template>
  <div>
    <b-container fluid>
      <b-row>
        <b-col xs="12" order-xs="2" lg="3">
          <b-row>
            <b-col class="mb-3">
              <b-form-input  v-model="search" placeholder="Search" block />
            </b-col>
          </b-row>
          <b-row class="text-left">
            <b-col>
              <router-link :to="{ name: 'AddProject' }">
                <b-button variant="primary" class="mb-1 add-new" block
                  ><b-row
                    ><b-col class="text-left">Add new project</b-col
                    ><b-col class="ml-auto text-right">
                      <b-icon-plus-circle /></b-col></b-row></b-button
              ></router-link>
              <router-link
                v-for="p in filtered_projects"
                :key="p.id"
                :to="{
                  name: `ProjectView`,
                  params: {
                    projectId: p.id,
                  },
                }"
              >
                <b-card :title="p.name" class="mb-1">
                  <b-card-text>
                    {{ p.description }}
                  </b-card-text>
                </b-card>
              </router-link>
            </b-col>
          </b-row>
        </b-col>
        <b-col lg="9">
          <router-view></router-view>
        </b-col>
      </b-row>
    </b-container>
  </div>
</template>

<style>
.add-new {
  height: 40px;
  font-size: large;
}
</style>
<script>
import axios from "axios";

export default {
  name: "Dashboard",
  components: {
  },
  data() {
    return {
      projects: [],
      filtered_projects: [],
      search: "",
    };
  },
  async mounted() {
    let user_id = $cookies.get("token");

    let response = await axios.get(
      `http://localhost:3001/api/projects/u/${user_id}`
    );

    this.projects = response.data.map((x) => {
      let y = {
        ...x,
        id: x._id.$oid,
        user_id: x.user_id.$oid,
        date_created: x.date_created.$date,
      };
      delete y._id;
      return y;
    });

    this.filtered_projects = this.projects;
  },
  methods: {},
  watch: {
    search: function() {
      if (this.search === "") {
        this.filtered_projects = this.projects;
      }
      else {
        this.filtered_projects = this.projects.filter((x) => {
          if (x['name'].includes(this.search)) {
            return x;
          }
        });
      }
    },
  }
};
</script>
