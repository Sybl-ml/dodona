<template>
  <b-navbar :key="$route.fullPath" toggleable="md">
    <b-navbar-brand :to="logoRoute">
      <icon-logo width="5em" height="3em" :show_text="true" />
    </b-navbar-brand>
    <b-navbar-toggle target="landingCollapse"> </b-navbar-toggle>
    <b-collapse is-nav id="landingCollapse" v-if="atDashboard">
    <b-navbar-nav>
      <b-nav-item disabled> {{ time }} </b-nav-item>
    </b-navbar-nav>
    </b-collapse>

    <b-collapse is-nav id="landingCollapse" v-else-if="atLanding">
      <b-navbar-nav>
        <b-nav-item>Product</b-nav-item>
        <b-nav-item>Meet the Team</b-nav-item>
        <b-nav-item to="/pricing">Pricing</b-nav-item>
        <b-nav-item>Guides</b-nav-item>
      </b-navbar-nav>
    </b-collapse>
    <b-collapse is-nav id="landingCollapse" v-if="loggedIn">
    <b-navbar-nav class="ml-auto" v-if="loggedIn">
      <b-nav-item disabled>{{ credits }} Credits</b-nav-item>
      <b-nav-item-dropdown right>
        <template #button-content>
          <b-avatar
            size="1.75em"
            :src="'data:image/png;base64,' + avatar"
          ></b-avatar>
          {{ name }}
        </template>
        <b-dropdown-item disabled>{{ email }}</b-dropdown-item>
        <b-dropdown-divider />
        <b-dropdown-item to="/dashboard">Dashboard</b-dropdown-item>
        <b-dropdown-item v-if="client" to="/nodes">Models</b-dropdown-item>
        <b-dropdown-item v-else to="/client/confirm">
          Register as Client
        </b-dropdown-item>
        <b-dropdown-divider />
        <b-dropdown-item to="/settings">My Profile</b-dropdown-item>
        <b-dropdown-item to="#">Help</b-dropdown-item>
        <b-dropdown-divider />
        <b-dropdown-item @click="signout">Sign Out</b-dropdown-item>
      </b-nav-item-dropdown>
    </b-navbar-nav>
    </b-collapse>

    <b-collapse is-nav id="landingCollapse" v-else>
      <b-navbar-nav class="ml-auto">
        <b-nav-form>
          <b-nav-item class="mr-1"
            ><router-link to="/login">Sign In</router-link></b-nav-item
          >
          <b-button variant="primary" to="/register">SIGN UP NOW</b-button>
        </b-nav-form>
      </b-navbar-nav>
    </b-collapse>
  </b-navbar>
</template>

<style>
.img-circle {
  height: 2rem;
  border-radius: 50%;
}
</style>

<script>
import IconLogo from "./icons/IconLogo";

export default {
  name: "Header",
  components: {
    IconLogo,
  },
  data() {
    return {
      name: "",
      email: "",
      client: false,
      time: "",
      credits: 0,
      loggedIn: false,
      logoRoute: "/",
      atDashboard: false,
      atLanding: false,
      avatar: "",
    };
  },
  methods: {
    signout: function () {
      $cookies.remove("token");
      this.$router.push("/login");
    },
    getUserData: async function () {
      let user_id = $cookies.get("token");
      if (!user_id) {
        return;
      }
      try {
        let user_data = await this.$http.get(`api/users`);
        this.name = user_data.data.first_name + " " + user_data.data.last_name;
        this.email = user_data.data.email;
        this.client = user_data.data.client;
        this.credits = user_data.data.credits;
      } catch (err) {
        console.log(err);
      }
    },
    updateHeader: function () {
      let user_id = $cookies.get("token");

      this.getUserData();

      let pageName = this.$route.name;

      this.loggedIn = user_id ? true : false;
      this.logoRoute = user_id ? "/dashboard" : "/";

      this.atLanding = pageName == "Welcome" || pageName == "Pricing";

      this.atDashboard =
        pageName === "Dashboard" ||
        pageName === "Settings" ||
        pageName === "ProjectView" ||
        pageName === "Nodes";
    },
    async getAvatar() {
      if ($cookies.get("token")) {
        let response = await this.$http.get(`api/users/avatar`);
        console.log(response);
        this.avatar = response.data.img;
      }
    },
  },
  async mounted() {
    this.getUserData();
    this.getAvatar();
    setInterval(() => {
      this.time = new Date().toLocaleString("en-GB", {
        dateStyle: "long",
        timeStyle: "medium",
      });
      this.time = this.time.toString().replace(" at", ",");
    }, 1000);
  },
  created() {
    this.updateHeader();
  },
  watch: {
    $route: function () {
      this.updateHeader();
    },
  },
};
</script>
